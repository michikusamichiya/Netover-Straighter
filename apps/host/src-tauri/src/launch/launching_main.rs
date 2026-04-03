use base64::Engine;
use base64::engine::general_purpose;
use serde::Deserialize;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::ice_transport::ice_candidate::RTCIceCandidateInit;
use webrtc::ice_transport::ice_connection_state::RTCIceConnectionState;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::data_channel::RTCDataChannel;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use std::collections::HashMap;
use tauri::{AppHandle, Manager};
use std::fs;
use rand::rngs::OsRng;
use rand::RngCore;
use crate::launch::ScreenOutputer::Capture::create_capture_way;
use crate::ws::wshandle::WsClient;
use webrtc::api::APIBuilder;
use webrtc::peer_connection::configuration::RTCConfiguration;
use crate::appstate::AppState;
use tauri::State;
use std::future::Future;
use std::{pin::Pin,sync::Arc};
use crate::ws::wshandle::Message;
use crate::emitter::emit_safer;
use tokio::sync::Mutex;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use crate::launch::InputInjector::types::InputInjector;
use crate::launch::InputInjector::types::InputStat;
use crate::launch::ScreenOutputer::types::Screen;
use crate::launch::ScreenOutputer::types::ScreenManager;
use webrtc::track::track_local::track_local_static_sample::TrackLocalStaticSample;
use webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability;
use webrtc::media::Sample;
use crate::launch::ScreenOutputer::types::CaptureLoop;
use crate::launch::ScreenOutputer::Encoder::create_encoder;
use crate::launch::ScreenOutputer::Capture::platform::windows::WindowsCaptureLoop;
use tungstenite::Bytes;
use webrtc::api::media_engine::MediaEngine;
use webrtc::rtp_transceiver::rtp_codec::{RTCRtpCodecParameters, RTPCodecType};
use webrtc::interceptor::registry::Registry;
use webrtc::api::interceptor_registry::register_default_interceptors;

type HmacSha256 = Hmac<Sha256>;
 
macro_rules! reporter {
    ($e:expr) => {
        log::error!("{}", $e)
    };
}
 
use crate::pairing::pairing_main::StoredKey;
 
#[derive(PartialEq, Debug)]
pub enum Status {
    Initializing, // 初期化中
    Pending, // 接続待ち
    Requested, // 要求受け取り
    Connecting, // 接続中
    InControl, // 接続完了, 操作中
    Error // エラー
}
pub struct LaunchingSession {
    pub config: RTCConfiguration,
    pub keys: HashMap<String, [u8; 32]>,
    pub ws: Option<WsClient>,
    pub rtc: Option<Arc<RTCSession>>,
    pub hmac: [u8; 32],
    pub stat: Status,
    pub pending_ice: Vec<serde_json::Value>,
    pub pending_answer: Option<RTCSessionDescription>,
    pub capture_loop: Option<Box<dyn CaptureLoop>>,

}
pub struct RTCSession {
    pc: Arc<RTCPeerConnection>,
    dc: Option<Arc<RTCDataChannel>>,
}
 
#[derive(Deserialize, Clone)]
pub struct Requested {
    keyid: String,
}
#[derive(Deserialize, Clone)]
pub struct GetSign {
    pub sign: Vec<u8>
}
#[derive(Deserialize, Clone)]
pub struct Answer {
    pub answer: String,
}
#[derive(Deserialize, Clone)]
pub struct IceCandidate {
    pub candidate: serde_json::Value,
}
 
#[derive(Deserialize)]
#[serde(tag = "type")]
enum LaunchingInfomation {
    #[serde(rename = "init_success")]
    InitSuccess,
    #[serde(rename = "requested")]
    Requested(Requested),
    #[serde(rename = "getsign")]
    GetSign(GetSign),
    #[serde(rename = "answer")]
    Answer(Answer),
    #[serde(rename = "ice-candidate")]
    IceCandidate(IceCandidate),
    #[serde(rename = "controller_disconnected")]
    ControllerDisconnected,
}
 
 
async fn loadkeys(app: AppHandle) -> Result<HashMap<String, [u8; 32]>, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|_| "Failed to get app data directory")?;
    let keys_file = app_data_dir.join("pairing_keys.json");
 
    if !keys_file.exists() {
        return Ok(HashMap::new());
    }
 
    let content = fs::read_to_string(&keys_file).map_err(|e| format!("Failed to read keys: {}", e))?;
    let mut keys: HashMap<String, StoredKey> = serde_json::from_str(&content).map_err(|e| format!("Failed to parse keys: {}", e))?;
 
    for (id, key) in keys.iter_mut() {
        if let Ok(entry) = keyring::Entry::new("netover-bloodway", &id) {
            if let Ok(password) = entry.get_password() {
                key.body = password;
            }
        }
    }
    let keys = keys.into_iter()
        .filter(|(_, k)| k.available); // 利用可能なもののみ取り出す
 
    let keys = keys.map(|(id, k)| { // Base64をu8; 32に変換することを試みる
        let decoded = general_purpose::STANDARD
            .decode(k.body)
            .map_err(|e| format!("Fatal: Failed to decode {} key. Details: {}. This should never happen. Please report this bug at GitHub issue and delete this key.", id, e))?;
        if decoded.len() != 32 {
            return Err(format!("Fatal: {} key is not 32 bytes long. This should never happen. Please report this bug at GitHub issue and delete this key.", id));
        }
        // Vec u8から変換
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&decoded);
        Ok((id, arr))
    }).collect::<Result<HashMap<String, [u8; 32]>, String>>()?;
    Ok(keys)
}
async fn loadkeys_onlyid(app: AppHandle) -> Result<Vec<String>, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|_| "Failed to get app data directory")?;
    let keys_file = app_data_dir.join("pairing_keys.json");
 
    if !keys_file.exists() {
        return Ok(vec![]);
    }
 
    let content = fs::read_to_string(&keys_file).map_err(|e| format!("Failed to read keys: {}", e))?;
    let keys: HashMap<String, StoredKey> = serde_json::from_str(&content).map_err(|e| format!("Failed to parse keys: {}", e))?;
 
    let keys = keys.iter().filter(|(_, k)| k.available).map(|(id, _)| id.clone()).collect();
    Ok(keys)
 
    // TODO: ゼロライズする
}
 
async fn handle_deny(launching: &Arc<Mutex<Option<LaunchingSession>>>) -> Result<(), String> {
    let mut guard = launching.lock().await;
    if let Some(session) = guard.as_mut() {
        session.stat = Status::Pending;
        session.rtc = None;
        session.pending_ice.clear();
        session.pending_answer = None;
        if let Some(ws) = session.ws.as_mut() {
            ws.send(Message::Message(serde_json::json!({
                "type": "deny"
            }).to_string())).await.map_err(|_| "Failed to send message")?;
        }
    }
    Ok(())
}
 
async fn handle_requested(
    launching: &Arc<Mutex<Option<LaunchingSession>>>,
    app: &AppHandle,
    requested: Requested,
) -> Result<(), String> {
    let mut guard = launching.lock().await;
    if let Some(session) = guard.as_mut() {
        if session.stat != Status::Pending {
            handle_deny(&launching).await?;
            return Ok(());
        }
        session.stat = Status::Requested;
        let mut nonce = [0u8; 32];
        OsRng.fill_bytes(&mut nonce);
 
        let key = session.keys.get(&requested.keyid).ok_or_else(|| format!("Key not found: {}", requested.keyid))?;
        let mut mac = HmacSha256::new_from_slice(&key[..])
            .expect("Failed to create hmac");
        mac.update(&nonce);
 
        let sign = mac.finalize().into_bytes();
        session.hmac = sign.into();
 
        if let Some(ws) = session.ws.as_mut() {
            ws.send(Message::Message(serde_json::json!({
                "type": "queryverify",
                "nonce": nonce.to_vec(),
            }).to_string())).await.map_err(|_| "Failed to send message")?;
        }
 
        emit_safer(&app, "launching_requested", "", |e| reporter!(e));
    }
    Ok(())
}
 
async fn create_peer_connection_and_send_offer(
    launching: &Arc<Mutex<Option<LaunchingSession>>>, 
    app: &AppHandle,
    config: RTCConfiguration
) -> Result<(Arc<RTCPeerConnection>, Arc<RTCDataChannel>), String> {
    let mut media_engine = MediaEngine::default();
    media_engine.register_codec(
        RTCRtpCodecParameters {
            capability: webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability {
                mime_type: "video/H264".to_string(),
                clock_rate: 90000,
                sdp_fmtp_line: "level-asymmetry-allowed=1;packetization-mode=1;profile-level-id=42001f".to_string(),
                rtcp_feedback: vec![
                    webrtc::rtp_transceiver::RTCPFeedback {
                        typ: "nack".to_string(),
                        parameter: "".to_string(),
                    },
                    webrtc::rtp_transceiver::RTCPFeedback {
                        typ: "nack".to_string(),
                        parameter: "pli".to_string(),
                    },
                ],
                ..Default::default()
            },
            payload_type: 102,
            ..Default::default()
        },
        RTPCodecType::Video,
    ).map_err(|e| format!("Failed to register codec: {}", e))?;

    let mut registry = Registry::new();
    registry = register_default_interceptors(registry, &mut media_engine)
        .map_err(|e| format!("Failed to register interceptors: {}", e))?;

    // NACKやPLIのフィードバックパケットを処理し、パケットロス時に即座に再送を行うインターセプターを有効化
    let api = APIBuilder::new()
        .with_media_engine(media_engine)
        .with_interceptor_registry(registry)
        .build();
    let peer_connection = Arc::new(
        api.new_peer_connection(config)
            .await
            .map_err(|e| format!("Failed to create peer connection: {}", e))?
    );
    
    let launching_for_ice = launching.clone();
    peer_connection.on_ice_candidate(Box::new(move |candidate| {
        let launching = launching_for_ice.clone();
        println!("ICE candidate: {:?}", candidate); // * デバッグ用
        Box::pin(async move {
            if let Some(c) = candidate {
                let json = match c.to_json() {
                    Ok(j) => j,
                    Err(_) => return,
                };
 
                // シグナリングで送信
                let guard = launching.lock().await;
                if let Some(session) = guard.as_ref() {
                    if let Some(ws) = session.ws.as_ref() {
                        let _ = ws.send(Message::Message(serde_json::json!({
                            "type": "ice-candidate",
                            "candidate": json,
                        }).to_string())).await;
                    }
                }
            }
        })
    }));
 
    // 接続断を検知してセッション全体を破棄する。
    // - Disconnected: ICEは一時的な状態であり自動回復の可能性があるため、
    //   5秒待機してから状態を再確認し、まだ Disconnected/Failed であれば終了する。
    // - Failed / Closed: 回復不可能なので即座に終了する。
    peer_connection.on_ice_connection_state_change(Box::new(move |cs| {
        Box::pin(async move {
            println!("ICE connection state changed: {:?}", cs);
        })
    }));
 
    // ① TrackをH264で作成
    // VP8から変更: HWエンコード移行時もここのmime_typeを変えるだけでよい
    let track = Arc::new(TrackLocalStaticSample::new(
        RTCRtpCodecCapability {
            mime_type: "video/H264".to_string(),
            ..Default::default()
        },
        "video".to_string(),
        "track1".to_string(),
    ));

    let dc = peer_connection.create_data_channel("operation", None).await
        .map_err(|e| format!("Failed to create data channel: {}", e))?;
        
    let launching_for_dc_close = launching.clone();
    let app_for_dc_close = app.clone();
    dc.on_close(Box::new(move || {
        let launching = launching_for_dc_close.clone();
        let app = app_for_dc_close.clone();
        Box::pin(async move {
            println!("Data channel closed");
            let should = {
                let guard = launching.lock().await;
                guard.as_ref().map(|s| s.stat != Status::Pending).unwrap_or(false)
            };
            if should {
                let _ = end_launching_inner(launching).await;
                emit_safer(&app, "launching_rtc_close", "", |e| reporter!(e));
            }
        })
    }));

    let launching_for_open = launching.clone();
    let app_for_open = app.clone();
    let track_for_open = track.clone();
    let handle = tokio::runtime::Handle::current();
    dc.on_open(Box::new(move || {
        let launching = launching_for_open.clone();
        let app = app_for_open.clone();
        let track = track_for_open.clone();
        let handle = handle.clone();
        println!("Data channel opened");
        Box::pin(async move {
            emit_safer(&app, "launching_rtc_open", "", |e| reporter!(e));
            
            let native_screen_opt = {
                let app_state = app.state::<AppState>();
                let capture_stat_guard = app_state.capture_stat.lock().await;
                if let Some(stat) = capture_stat_guard.as_ref() {
                    stat.native_map.get(&stat.now_screen.id).cloned()
                } else {
                    None
                }
            };
            
            tauri::async_runtime::spawn(async move {
                let mut cl_box: Option<Box<dyn CaptureLoop>> = None;
                if let Some(native_screen) = native_screen_opt {
                    if let Ok(encoder) = create_encoder() {
                        let track_for_capture = track.clone();
                        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();
                        
                        tokio::spawn(async move {
                            let mut last_frame_time = std::time::Instant::now();
                            while let Some(encoded) = rx.recv().await {
                                let now = std::time::Instant::now();
                                let mut duration = now.duration_since(last_frame_time);
                                last_frame_time = now;
                                
                                if duration.as_millis() == 0 {
                                    duration = std::time::Duration::from_millis(1);
                                }

                                let sample = webrtc::media::Sample {
                                    data: Bytes::from(encoded),
                                    duration,
                                    ..Default::default()
                                };
                                let _ = track_for_capture.write_sample(&sample).await;
                            }
                        });

                        let on_frame: Box<dyn Fn(Vec<u8>) + Send> = Box::new(move |encoded| {
                            let _ = tx.send(encoded);
                        });

                        let mut capture_loop = WindowsCaptureLoop::new();
                        if capture_loop.start(&native_screen, encoder, on_frame).is_ok() {
                            cl_box = Some(Box::new(capture_loop));
                        }
                    }
                }

                let mut guard = launching.lock().await;
                if let Some(session) = guard.as_mut() {
                    session.stat = Status::InControl;
                    if let Some(cl) = cl_box {
                        session.capture_loop = Some(cl);
                    }
                }
            });
        })
    }));

    let app_handle = app.clone();
    dc.on_message(Box::new(move |msg| {
        let app = app_handle.clone();
        let msg_str = String::from_utf8_lossy(&msg.data).to_string();
        Box::pin(async move {
            tauri::async_runtime::spawn(async move {
                let state = app.state::<AppState>();
                let input_trait = state.input_trait.clone();
                let input_stat_mutex = state.input_stat.clone();
     
                let stat_guard = input_stat_mutex.lock().await;
                if let Some(stat) = stat_guard.as_ref() {
                    let res = input_trait.handle_input(stat, msg_str).await;
                    if let Err(err) = res {
                        println!("Failed to handle input: {:?}", err);
                    }
                }
            });
        })
    }));

    // ② PeerConnectionにTrackを登録
    let track_dyn = Arc::clone(&track) as Arc<dyn webrtc::track::track_local::TrackLocal + Send + Sync>;
    let rtp_sender = peer_connection.add_track(track_dyn).await
        .map_err(|e| format!("Failed to add track: {}", e))?;

    // RTCPパケットを読み捨てるループ（これがないとインターセプタが詰まったりPLI等が処理されない）
    tokio::spawn(async move {
        let mut rtcp_buf = vec![0u8; 1500];
        while let Ok((_, _)) = rtp_sender.read(&mut rtcp_buf).await {}
    });

    // キャプチャの起動は DataChannel が Open したタイミングに遅延させる
    // これにより最初のIDRフレームが確実にブラウザに届くようになる

    // オファーの作成、ローカル側セット
    let offer = peer_connection.create_offer(None)
        .await.map_err(|e| format!("Failed to create offer: {}", e))?;
    peer_connection.set_local_description(offer.clone())
        .await.map_err(|e| format!("Failed to set local description: {}", e))?;
 
    let guard = launching.lock().await;
    if let Some(session) = guard.as_ref() {
        if let Some(ws) = session.ws.as_ref() {
            ws.send(Message::Message(serde_json::json!({
                "type": "accept-offer",
                "sdp": offer.sdp,
            }).to_string())).await.map_err(|_| "Failed to send message")?;
        }
    }

    Ok((peer_connection, dc.clone()))
}
 
async fn handle_ice_candidate(
    launching: &Arc<Mutex<Option<LaunchingSession>>>,
    ice_candidate: IceCandidate,
) -> Result<(), String> {
    let rtc_opt = {
        let mut guard = launching.lock().await;
        if let Some(session) = guard.as_mut() {
            if let Some(rtc) = session.rtc.as_ref() {
                Some(rtc.clone())
            } else {
                session.pending_ice.push(ice_candidate.candidate.clone());
                None
            }
        } else {
            None
        }
    };

    if let Some(rtc) = rtc_opt {
        if rtc.pc.remote_description().await.is_some() {
            let c: RTCIceCandidateInit = serde_json::from_value(ice_candidate.candidate.clone())
                .map_err(|e| format!("Failed to parse ice candidate: {}", e))?;
            rtc.pc.add_ice_candidate(c).await
                .map_err(|e| format!("Failed to add ice candidate: {}", e))?;
        } else {
            let mut guard = launching.lock().await;
            if let Some(session) = guard.as_mut() {
                session.pending_ice.push(ice_candidate.candidate.clone());
            }
        }
    }
    Ok(())
}
 
async fn handle_queryverify_and_send_offer(
    launching: &Arc<Mutex<Option<LaunchingSession>>>, 
    app: &AppHandle, 
    queryverify: GetSign
) -> Result<(), String> {
    let (config, ok) = {
        let guard = launching.lock().await;
        let session = match guard.as_ref() {
            Some(s) => s,
            None => return Ok(()),
        };
 
        let status_ok = session.stat == Status::Requested;
        let sign_ok = queryverify.sign.as_slice() == &session.hmac[..];
        (session.config.clone(), status_ok && sign_ok)
    };
 
    if !ok {
        handle_deny(&launching).await?;
        return Ok(());
    }
 
    // 検証OK後に接続を開始
    emit_safer(&app, "launching_queryverify", "", |e| reporter!(e));
 
    // ここから接続
    let (peer_connection, data_channel) =
        create_peer_connection_and_send_offer(&launching, &app, config).await?;
 
    let pending_answer = {
        let mut guard = launching.lock().await;
        let session = match guard.as_mut() {
            Some(s) => s,
            None => return Ok(()),
        };
        session.stat = Status::Connecting;
        session.rtc = Some(Arc::new(RTCSession {
            pc: peer_connection,
            dc: Some(data_channel),
        }));
        session.pending_answer.take()
    };
 
    // answer が先に届いていた場合は、ここで適用する
    if let Some(ans) = pending_answer {
        let _ = handle_answer_and_set_remote_description(&launching, &app, ans).await;
    }
    Ok(())
}
 
async fn handle_answer_and_set_remote_description(
    launching: &Arc<Mutex<Option<LaunchingSession>>>, 
    app: &AppHandle, 
    answer: RTCSessionDescription
) -> Result<(), String> {
 
    // 🔥 必要なものだけ取り出して即unlock
    let (rtc, pending_ice) = {
        let mut guard = launching.lock().await;
 
        let session = guard
            .as_mut()
            .ok_or_else(|| "Session not found".to_string())?;
 
        let rtc = session.rtc.clone();
        if rtc.is_none() {
            // RTCがまだ準備できていない（answerが先に届いた）場合は保留する
            session.pending_answer = Some(answer);
            return Ok(());
        }
 
        let pending_ice = std::mem::take(&mut session.pending_ice);
        (rtc.unwrap(), pending_ice)
    }; // ← ここでlock解放
 
    // 🔥 lockなしでawait（超重要）
    rtc.pc
        .set_remote_description(answer)
        .await
        .map_err(|e| format!("Failed to set remote description: {}", e))?;
 
    // 早着 ICE をここでまとめて投入
    for ice in pending_ice {
        let c: RTCIceCandidateInit = serde_json::from_value(ice)
            .map_err(|e| format!("Failed to parse ice candidate: {}", e))?;
        rtc.pc
            .add_ice_candidate(c)
            .await
            .map_err(|e| format!("Failed to add ice candidate: {}", e))?;
    }
 
    emit_safer(app, "launching_answer", "", |e| reporter!(e));
 
    Ok(())
}
 
async fn end_launching_inner(launching: Arc<Mutex<Option<LaunchingSession>>>) -> Result<(), String> {
    let (ws, rtc) = {
        let mut guard = launching.lock().await;
        // capture_loopも止める
        if let Some(session) = guard.as_mut() {
            if let Some(cl) = session.capture_loop.as_mut() {
                cl.stop();
            }
        }
        let ws = guard.as_mut().and_then(|s| s.ws.take());
        let rtc = guard.as_mut().and_then(|s| s.rtc.take());
        (ws, rtc)
    };
 
    if let Some(ws) = ws {
        let _ = ws.close().await;
    }
    
    if let Some(rtc) = rtc {
        let _ = rtc.pc.close().await;
    }
 
    let mut guard = launching.lock().await;
    *guard = None;
 
    Ok(())
}
 
#[tauri::command]
pub async fn launch(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let app2 = app.clone();
    let launching_state = state.launching.clone();
 
    let config = RTCConfiguration {
        ice_servers: vec![
            RTCIceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_string()],
                ..Default::default()
            },
        ],
        ..Default::default()
    };
 
    let handle_message: Arc<
        dyn Fn(Message) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>> + Send + Sync + 'static
    > = Arc::new(move |msg| {
        let app = app2.clone();
        let state = launching_state.clone();
        
        Box::pin(async move {
            match msg {
                Message::Message(txt) => {
                    println!("{}", txt);
                    let data = match serde_json::from_str::<LaunchingInfomation>(&txt) {
                        Ok(data) => data,
                        Err(e) => {
                            emit_safer(&app, "launching_error", e.to_string(), |e| reporter!(e));
                            return;
                        }
                    };
                    match data {
                        LaunchingInfomation::InitSuccess => {
                            emit_safer(&app, "launching_init_success", "", |e| reporter!(e));
                            // TODO: キャプチャ
                        }
                        LaunchingInfomation::Requested(requested) => {
                            match handle_requested(&state, &app, requested)
                                .await
                                .map_err(|e| reporter!(e)) 
                            {
                                Ok(_) => {},
                                Err(e) => {
                                    emit_safer(&app, "launching_error", e, |e| reporter!(e));
                                }
                            }
                        }
                        LaunchingInfomation::GetSign(sign) => {
                            match handle_queryverify_and_send_offer(&state, &app, sign)
                                .await
                                .map_err(|e| reporter!(e))
                            {
                                Ok(_) => {
                                    emit_safer(&app, "launching_getsign", "", |e| reporter!(e));
                                }
                                Err(e) => {
                                    emit_safer(&app, "launching_error", e, |e| reporter!(e));
                                }
                            }
                        }
                        LaunchingInfomation::Answer(answer) => {
                            let answer_desc = match RTCSessionDescription::answer(answer.answer) {
                                Ok(d) => d,
                                Err(e) => {
                                    emit_safer(&app, "launching_error", e.to_string(), |e2| reporter!(e2));
                                    return;
                                }
                            };
 
                            match handle_answer_and_set_remote_description(&state, &app, answer_desc).await {
                                Ok(_) => {},
                                Err(e) => {
                                    emit_safer(&app, "launching_error", e, |e| reporter!(e));
                                }
                            }
                        }
                        LaunchingInfomation::IceCandidate(ice_candidate) => {
                            match handle_ice_candidate(&state, ice_candidate).await {
                                Ok(_) => {
                                    emit_safer(&app, "launching_ice_candidate", "", |e| reporter!(e));
                                }
                                Err(e) => {
                                    emit_safer(&app, "launching_error", e, |e| reporter!(e));
                                }
                            }
                        }
                        LaunchingInfomation::ControllerDisconnected => {
                            // コントローラーが切断されたのでRTCセッションをリセットし、
                            // 再度接続を受け付けられる Pending 状態に戻す
                            let mut guard = state.lock().await;
                            if let Some(session) = guard.as_mut() {
                                if let Some(cl) = session.capture_loop.as_mut() {
                                    cl.stop();
                                }
                                session.capture_loop = None;
                                session.rtc = None;
                                session.pending_ice.clear();
                                session.pending_answer = None;
                                session.stat = Status::Pending;
                            }
                            emit_safer(&app, "launching_controller_disconnected", "", |e| reporter!(e));
                        }
                    }
                }
                Message::Close => {
                    match end_launching_inner(state).await {
                        Ok(_) => {}
                        Err(e) => {
                            emit_safer(&app, "launching_error", e, |e| reporter!(e));
                        }
                    }
                }
                Message::Connected => {
                    
                }
                Message::Error(e) => {
                    emit_safer(
                        &app,
                        "launching_error",
                        format!("A fatal error has been occurred in WebSocket connection. Details: {}", e),
                        |e| reporter!(e),
                    );
                    match end_launching_inner(state).await {
                        Ok(_) => {}
                        Err(e) => {
                            emit_safer(&app, "launching_error", e, |e| reporter!(e));
                        }
                    }
                }
            }
        })
    });
 
    {
        // get_screens()は一度だけ呼ぶ
        let screens = state.capture_trait.get_screens().map_err(|_| "Failed to get screens, WinAPI ERROR")?;
 
        // プライマリ画面を探す。なければ先頭をフォールバックとする
        // TODO: スクリーンが0件の場合のハンドリングは未完成
        let now_screen = screens
            .iter()
            .find(|(s, _)| s.primary)
            .or_else(|| screens.first())
            .ok_or("No screens found")?
            .0
            .clone(); // Screen を clone して所有権を得る
 
        // スクリーンの対応表を作成（screensを消費しないようにiter()で回す）
        let mut native_map = HashMap::new();
        for (screen, native_screen) in &screens {
            native_map.insert(screen.id.clone(), native_screen.clone());
        }
 
        // screensはVec<(Screen, NativeScreen)>なのでScreen部分だけ取り出す
        let screen_list: Vec<Screen> = screens.into_iter().map(|(s, _)| s).collect();
 
        let mut guard = state.capture_stat.lock().await;
        *guard = Some(ScreenManager {
            screens: screen_list,
            now_screen: now_screen.clone(),
            native_map,
        });
    }
    {
        let now_screen = {
            let guard = state.capture_stat.lock().await;
            guard
                .as_ref()
                .ok_or("capture_stat is not initialized")?
                .now_screen
                .clone()
        };
        let mut guard = state.input_stat.lock().await;
        *guard = Some(InputStat {
            screen: now_screen,
            keymap: Arc::new(state.input_trait.load_keymap().map_err(|_| "Failed to load keymap")?),
        });
    }
 
    let mut guard = state.launching.lock().await;
    if guard.is_some() {
        return Err("Launching session already running".to_string());
    }
 
    let keys = loadkeys(app.clone()).await?;
    let app_config = crate::config::config_main::get_config(app.clone()).unwrap_or_default();
    let url = format!("{}/ws/launch/target", app_config.server_url);
    let ws = WsClient::new(&url, handle_message)
        .await
        .map_err(|e| e.to_string())?;
 
    let msg = serde_json::json!({
        "type": "init",
        "keys": loadkeys_onlyid(app.clone()).await?
    })
    .to_string();
    ws.send(Message::Message(msg))
        .await
        .map_err(|_| "Failed to send init")?;
 
    *guard = Some(LaunchingSession {
        config,
        keys,
        ws: Some(ws),
        rtc: None,
        hmac: [0u8; 32],
        stat: Status::Pending,
        pending_ice: Vec::new(),
        pending_answer: None,
        capture_loop: None,
    });
 
    emit_safer(&app, "launching_connected", "", |e| reporter!(e));
 
    Ok(())
}
 
#[tauri::command]
pub async fn end_launching(state: State<'_, AppState>) -> Result<(), String> {
    end_launching_inner(state.launching.clone()).await
}