use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::connect_async;
use tokio::sync::mpsc::{self, UnboundedSender, UnboundedReceiver};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use tokio_tungstenite::tungstenite::Message as WsMsg;
use crate::ws::def::MessageHandler;

#[derive(Clone, Debug)]
pub enum Message {
    Message(String),
    Close,
    Connected,
    Error(String),
}

// ハンドラは Arc + dyn Fn + Send + Sync で安全に共有
pub struct WsClient {
    tx: UnboundedSender<Message>,
    closed: Arc<AtomicBool>,
    rx: UnboundedReceiver<Message>,
    handler: MessageHandler,  
}

impl WsClient {
    pub async fn new(
        url: &str,
        handler: MessageHandler,
    ) -> Result<Self, &'static str> {
        let (ws_stream, _) = connect_async(url).await.map_err(|_| "ConnectionFailed")?;
        let (mut write, mut read) = ws_stream.split();

        // メッセージ送受信用チャネル
        let (tx, mut rx) = mpsc::unbounded_channel();
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        let closed = Arc::new(AtomicBool::new(false));

        // 書き込みタスク
        let write_closed = closed.clone();
        let event_tx_clone = event_tx.clone();
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                let ws_msg = match msg {
                    Message::Message(txt) => {
                        log::info!("WS Send: {}", txt);
                        WsMsg::Text(txt.into())
                    }
                    Message::Close => {
                        log::info!("WS Send: [Close]");
                        WsMsg::Close(None)
                    }
                    _ => continue,
                };
                if write.send(ws_msg).await.is_err() { break; }
            }
            let _ = event_tx_clone.send(Message::Close);
            write_closed.store(true, Ordering::SeqCst);
        });

        // 読み取りタスク + ハンドラ呼び出し
        let read_closed = closed.clone();
        let handler_clone = handler.clone();
        let event_tx_clone2 = event_tx.clone();
        tokio::spawn(async move {
            let _ = event_tx_clone2.send(Message::Connected);
            (handler_clone)(Message::Connected).await;

            while let Some(msg) = read.next().await {
                let app_msg = match msg {
                    Ok(WsMsg::Text(txt)) => {
                        log::info!("WS Recv: {}", txt);
                        Message::Message(txt.to_string())
                    }
                    Ok(WsMsg::Binary(_)) => {
                        log::info!("WS Recv: [Binary data]");
                        continue;
                    }
                    Ok(WsMsg::Close(cl)) => {
                        log::info!("WS Recv: [Close, {:?}]", cl);
                        continue;
                    }
                    Ok(m) => {
                        log::info!("WS Recv: [Other: {:?}]", m);
                        continue;
                    }
                    Err(e) => {
                        log::error!("WS Error: {}", e);
                        Message::Error(e.to_string())
                    }
                };

                let _ = event_tx_clone2.send(app_msg.clone()); // イベントチャネル
                (handler_clone)(app_msg).await; // ハンドラ呼び出し
            }

            let _ = event_tx_clone2.send(Message::Close);
            (handler_clone)(Message::Close).await;
            read_closed.store(true, Ordering::SeqCst);
        });

        Ok(Self { tx, closed, rx: event_rx, handler })
    }

    pub async fn send(&self, msg: Message) -> Result<(), &'static str> {
        println!("WS Send: {:?}", msg);
        if self.closed.load(Ordering::SeqCst) { return Err("AlreadyClosed"); }
        self.tx.send(msg).map_err(|_| "SendFailed")?;
        Ok(())
    }

    pub async fn close(&self) -> Result<(), &'static str> {
        if self.closed.swap(true, Ordering::SeqCst) { return Err("AlreadyClosed"); }
        let _ = self.tx.send(Message::Close);
        Ok(())
    }
}