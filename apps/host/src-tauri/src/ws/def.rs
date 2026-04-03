use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use crate::ws::wshandle::Message;

pub type MessageHandler =
  Arc<dyn Fn(Message) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

#[derive(Debug)]
pub enum WebSocketSessionError {
  NotSetuped,
  InvalidUrl,
  ConnectionFailed,
  AlreadyClosed,
  SendFailed
}