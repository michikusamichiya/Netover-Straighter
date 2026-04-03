use crate::crypto::crypto::{self, compute_shared, generate_x25519};
use crate::ws::wshandle::Message;
use crate::ws::wshandle::WsClient;
use crate::{appstate::AppState, emitter::emit_safer};
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use std::fs;
use std::future::Future;
use std::{pin::Pin, sync::Arc};
use tauri::Manager;
use tauri::{AppHandle, State};
use tokio::sync::Mutex;
use x25519_dalek::{EphemeralSecret, PublicKey};

macro_rules! reporter {
    ($e:expr) => {
        log::error!("{}", $e)
    };
}

pub struct PairingSession {
    #[allow(dead_code)]
    pub host_id: Option<String>,
    pub host_public_key: Option<PublicKey>,
    pub host_private_key: Option<EphemeralSecret>,
    pub shared_secret: Option<[u8; 32]>,
    pub client_public_key: Option<PublicKey>,
    pub accepted: bool,
    pub ws: Option<WsClient>,
}

impl PairingSession {
    pub fn new() -> Self {
        Self {
            host_id: None,
            host_public_key: None,
            host_private_key: None,
            shared_secret: None,
            client_public_key: None,
            accepted: false,
            ws: None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct RandId {
    id: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct ExchangeFromLocal {
    #[serde(rename = "pub")]
    r#pub: Vec<u8>,
    local_id: String,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum PairingInfomation {
    #[serde(rename = "rand-id")] // ! ここだけハイフンなのは仕様だが、いつか修正する。サーバーと合わせて。
    RandId(RandId),

    #[serde(rename = "allowed_key")]
    AllowedKey,

    #[serde(rename = "accept_from_local")]
    AcceptFromLocal,

    #[serde(rename = "cancel_from_local")]
    CancelFromLocal,

    #[serde(rename = "deny_from_local")]
    DenyFromLocal,

    #[serde(rename = "disconnected_from_local")]
    DisconnectedFromLocal,

    #[serde(rename = "exchange_from_local")]
    ExchangeFromLocal(ExchangeFromLocal),
}

// ------- Inner Functions ------- //

async fn handle_save_id(
    id: String,
    pairing: &Arc<Mutex<Option<PairingSession>>>,
    app: &AppHandle,
    rand_id: RandId,
) {
    let mut guard = pairing.lock().await;
    if let Some(session) = guard.as_mut() {
        session.host_id = Some(id);
    }
    emit_safer(&app, "pairing_rand_id", rand_id.id, |e| reporter!(e));
}

async fn handle_exchange(
    pairing: &Arc<Mutex<Option<PairingSession>>>,
    app: &AppHandle,
    exchange_from_local: ExchangeFromLocal,
) -> Result<(), String> {
    let mut guard = pairing.lock().await;
    if let Some(session) = guard.as_mut() {
        let bytes: [u8; 32] = exchange_from_local
            .r#pub
            .as_slice()
            .try_into()
            .map_err(|_| "public key must be 32 bytes")?;

        session.client_public_key = Some(PublicKey::from(bytes));

        if let (Some(remote_privatekey), Some(local_publickey)) = (
            session.host_private_key.take(),
            session.client_public_key,
        ) {
            let shared_secret = compute_shared(remote_privatekey, local_publickey);
            session.shared_secret = Some(shared_secret);
        } else {
            return Err("Failed to compute shared secret".to_string());
        }

        // ここでshared_secretが計算されていることが証明されるため、unwrapしてもよい。
        let hash = crypto::bytes_to_string(&session.shared_secret.unwrap())[..16].to_string();

        emit_safer(&app, "pairing_exchange_from_local", hash, |e| reporter!(e));
    }
    Ok(())
}

async fn handle_allowed_key(
    // pairing: &Arc<Mutex<Option<PairingSession>>>,
    app: &AppHandle,
) -> Result<(), String> {
    // let mut guard = pairing.lock().await;
    // if let Some(session) = guard.as_mut() {
    //     if let Some(ws) = session.ws.as_mut() {
    //         let msg = serde_json::json!({"type": "allowed_key"}).to_string();
    //         ws.send(Message::Message(msg))
    //             .await
    //             .map_err(|_| "Failed to send allowed_key")?;
    //     }
    // }
    emit_safer(app, "pairing_allowed_key", "", |e| reporter!(e));
    Ok(())
}


async fn handle_accept_from_local(
    pairing: &Arc<Mutex<Option<PairingSession>>>,
    app: &AppHandle,
) -> Result<(), String> {
    let mut guard = pairing.lock().await;
    if let Some(session) = guard.as_mut() {
        session.accepted = true;
    }
    emit_safer(app, "pairing_accept_from_local", "", |e| reporter!(e));
    Ok(())
}
async fn handle_cancel_from_local(
    pairing: &Arc<Mutex<Option<PairingSession>>>,
    app: &AppHandle,
) -> Result<(), String> {
    let mut guard = pairing.lock().await;
    if let Some(session) = guard.as_mut() {
        session.accepted = false;
    }
    emit_safer(app, "pairing_cancel_from_local", "", |e| reporter!(e));
    Ok(())
}
async fn handle_deny_from_local(
    pairing: &Arc<Mutex<Option<PairingSession>>>,
    app: &AppHandle,
) -> Result<(), String> {
    let mut guard = pairing.lock().await;
    if let Some(session) = guard.as_mut() {
        session.accepted = false;
        if let Some(ws) = session.ws.as_mut() {
            ws.close().await.map_err(|_| "Failed to close ws")?;
        }
    }
    emit_safer(app, "pairing_deny_from_local", "", |e| reporter!(e));
    Ok(())
}
async fn handle_disconnected_from_local(app: &AppHandle) -> Result<(), String> {
    emit_safer(app, "pairing_disconnected_from_local", "", |e| reporter!(e));
    Ok(())
}

#[tauri::command]
pub async fn accept(state: State<'_, AppState>) -> Result<(), String> {
    let mut guard = state.pairing.lock().await;
    if let Some(session) = guard.as_mut() {
        if let Some(ws) = session.ws.as_mut() {
            let msg = serde_json::json!({"type": "accept"}).to_string();
            ws.send(Message::Message(msg))
                .await
                .map_err(|_| "Failed to send accept signal")?;
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn deny(state: State<'_, AppState>) -> Result<(), String> {
    let mut guard = state.pairing.lock().await;
    if let Some(session) = guard.as_mut() {
        if let Some(ws) = session.ws.as_mut() {
            let msg = serde_json::json!({"type": "deny"}).to_string();
            ws.send(Message::Message(msg))
                .await
                .map_err(|_| "Failed to send deny signal")?;
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn cancel(state: State<'_, AppState>) -> Result<(), String> {
    let mut guard = state.pairing.lock().await;
    if let Some(session) = guard.as_mut() {
        if let Some(ws) = session.ws.as_mut() {
            let msg = serde_json::json!({"type": "cancel"}).to_string();
            ws.send(Message::Message(msg))
                .await
                .map_err(|_| "Failed to send cancel signal")?;
        }
    }
    Ok(())
}

async fn end_pairing_inner(pairing: Arc<Mutex<Option<PairingSession>>>) -> Result<(), String> {
    let ws = {
        let mut guard = pairing.lock().await;
        guard.as_mut().and_then(|s| s.ws.take())
    };

    if let Some(ws) = ws {
        ws.close().await.map_err(|_| "Failed to close websocket")?;
    }

    let mut guard = pairing.lock().await;
    *guard = None;

    Ok(())
}

#[tauri::command]
pub async fn end_pairing(state: State<'_, AppState>) -> Result<(), String> {
    end_pairing_inner(state.pairing.clone()).await
}
// async fn handle_closed(pairing: &Arc<Mutex<Option<PairingSession>>>, app: &AppHandle) -> Result<(), String> {
//     emit_safer(app, "pairing_closed", "", |e| reporter!(e));
//     Ok(())
// }

// ------- Command ------- //

#[tauri::command]
pub async fn start_pairing(state: State<'_, AppState>, app: AppHandle) -> Result<(), String> {
    // Stateは短命なのでArcだけ取り出して持たせる
    let app2 = app.clone();
    let pairing_state = state.pairing.clone();

    // メッセージハンドラ
    let handle_message: Arc<
        dyn Fn(Message) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync,
    > = Arc::new(move |msg: Message| {
        let app = app2.clone();
        let state = pairing_state.clone();
        Box::pin(async move {
            match msg {
                Message::Message(txt) => {
                    let data = match serde_json::from_str::<PairingInfomation>(&txt) {
                        Ok(d) => d,
                        Err(e) => {
                            emit_safer(&app, "pairing_error", e.to_string(), |e| reporter!(e));
                            return;
                        }
                    };
                    let pairing = state.clone();
                    match data {
                        PairingInfomation::RandId(rand_id) => {
                            handle_save_id(rand_id.clone().id, &pairing, &app, rand_id).await;
                        }
                        PairingInfomation::ExchangeFromLocal(exchange_from_local) => {
                            match handle_exchange(&pairing, &app, exchange_from_local)
                                .await
                                .map_err(|e| reporter!(e))
                            {
                                Ok(_) => {}
                                Err(e) => {
                                    emit_safer(&app, "pairing_error", e, |e| reporter!(e));
                                }
                            }
                        }
                        PairingInfomation::AllowedKey => {
                            match handle_allowed_key(&app)
                                .await
                                .map_err(|e| reporter!(e))
                            {
                                Ok(_) => {}
                                Err(e) => {
                                    emit_safer(&app, "pairing_error", e, |e| reporter!(e));
                                }
                            }
                        }
                        PairingInfomation::AcceptFromLocal => {
                            match handle_accept_from_local(&pairing, &app)
                                .await
                                .map_err(|e| reporter!(e))
                            {
                                Ok(_) => {}
                                Err(e) => {
                                    emit_safer(&app, "pairing_error", e, |e| reporter!(e));
                                }
                            }
                        }
                        PairingInfomation::CancelFromLocal => {
                            match handle_cancel_from_local(&pairing, &app)
                                .await
                                .map_err(|e| reporter!(e))
                            {
                                Ok(_) => {}
                                Err(e) => {
                                    emit_safer(&app, "pairing_error", e, |e| reporter!(e));
                                }
                            }
                        }
                        PairingInfomation::DenyFromLocal => {
                            match handle_deny_from_local(&pairing, &app)
                                .await
                                .map_err(|e| reporter!(e))
                            {
                                Ok(_) => {}
                                Err(e) => {
                                    emit_safer(&app, "pairing_error", e, |e| reporter!(e));
                                }
                            }
                        }
                        PairingInfomation::DisconnectedFromLocal => {
                            match handle_disconnected_from_local(&app)
                                .await
                                .map_err(|e| reporter!(e))
                            {
                                Ok(_) => {}
                                Err(e) => {
                                    emit_safer(&app, "pairing_error", e, |e| reporter!(e));
                                }
                            }
                        }
                        _ => {
                            // Ignored
                        }
                    }
                }
                Message::Error(_) => {
                    emit_safer(
                        &app,
                        "pairing_error",
                        "A fatal error has been occurred in WebSocket connection.",
                        |e| reporter!(e),
                    );
                    match end_pairing_inner(state).await {
                        Ok(_) => {}
                        Err(e) => {
                            emit_safer(&app, "pairing_error", e, |e| reporter!(e));
                        }
                    }
                }
                Message::Connected => {
                    // 接続イベントは start_pairing 側で同期的に処理し、競合を避けるためここでは何もしない
                }
                Message::Close => {
                    emit_safer(
                        &app,
                        "pairing_disconnected",
                        "Disconnected from server",
                        |e| reporter!(e),
                    );
                }
                _ => {}
            }
        })
    });

    let mut session = PairingSession::new();
    
    // 鍵生成とセット
    let (secret, public) = generate_x25519();
    session.host_private_key = Some(secret);
    session.host_public_key = Some(public);

    let config = crate::config::config_main::get_config(app.clone()).unwrap_or_default();
    let url = format!("{}/ws/pairing/remote", config.server_url);
    let ws = match WsClient::new(&url, handle_message).await {
        Ok(ws) => ws,
        Err(_) => {
            emit_safer(
                &app,
                "pairing_error",
                "Failed to connect server. Please check network connection.",
                |e| reporter!(e),
            );
            return Err("Failed to connect server. Please check network connection.".to_string());
        }
    };
    session.ws = Some(ws);
    let mut pairing = state.pairing.lock().await;
    *pairing = Some(session);

    // Explicitly send init_remote and emit pairing_connected after everything is properly set up in state.
    if let Some(session) = pairing.as_mut() {
        let pub_key = session.host_public_key.as_ref().unwrap().as_bytes().to_vec();
        if let Some(ws) = session.ws.as_mut() {
            let msg = serde_json::json!({
                "type": "init_remote",
                "pub": pub_key
            }).to_string();
            ws.send(Message::Message(msg))
                .await
                .map_err(|_| "Failed to send init_remote")?;
        }
    }
    emit_safer(&app, "pairing_connected", "", |e| reporter!(e));

    Ok(())
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StoredKey {
    pub available: bool,
    pub body: String,
}

#[tauri::command]
pub async fn save_key(
    state: State<'_, AppState>,
    app: AppHandle,
    remote_id: String,
) -> Result<(), String> {
    let mut guard = state.pairing.lock().await;

    let (shared_secret, host_pub, client_pub) = if let Some(session) = guard.as_ref() {
        if let (Some(ss), Some(hp), Some(cp)) = (
            session.shared_secret,
            session.host_public_key,
            session.client_public_key,
        ) {
            (ss, hp, cp)
        } else {
            return Err("Shared secret or public keys not found in session".to_string());
        }
    } else {
        return Err("No active pairing session found".to_string());
    };

    // HKDF derivation
    let salt = crypto::derive_salt(host_pub.as_bytes(), client_pub.as_bytes());
    let okm = crypto::derive_hkdf_key(&shared_secret, &salt, b"netover-hmac-key-v1");
    let base64_key = general_purpose::STANDARD.encode(okm);

    // Persistence
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|_| "Failed to get app data directory")?;
    if !app_data_dir.exists() {
        fs::create_dir_all(&app_data_dir).map_err(|e| format!("Failed to create AppData: {}", e))?;
    }
    let keys_file = app_data_dir.join("pairing_keys.json");

    let mut keys: std::collections::HashMap<String, StoredKey> = if keys_file.exists() {
        let content = fs::read_to_string(&keys_file).map_err(|e| format!("Failed to read keys: {}", e))?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        std::collections::HashMap::new()
    };

    let entry = keyring::Entry::new("netover-bloodway", &remote_id)
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;
    entry.set_password(&base64_key)
        .map_err(|e| format!("Failed to save key in keyring: {}", e))?;

    keys.insert(
        remote_id,
        StoredKey {
            available: true,
            body: "".to_string(),
        },
    );

    let new_content =
        serde_json::to_string_pretty(&keys).map_err(|e| format!("Failed to serialize keys: {}", e))?;
    fs::write(&keys_file, new_content).map_err(|e| format!("Failed to write keys: {}", e))?;


    emit_safer(&app, "pairing_complete", "", |e| reporter!(e));
    // TODO: ゼロライズする

    Ok(())
}
