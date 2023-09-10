//! do NOT share this file in anyway possible
//!
//! No one can know that this file exist
//!
//! Every methods must be protected and need to ensure a proper identification before, E-V-E-R-Y-T-I-M-E

use std::sync::Arc;
use std::time::Duration;
use axum::extract::{State, WebSocketUpgrade};
use axum::extract::ws::{close_code, CloseFrame, Message, WebSocket};
use axum::response::Response;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, RwLock};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde_json::{json, Value};
use sha3::{ Digest, Sha3_512 };
use rand::{distributions::Alphanumeric, Rng};
use crate::AppState;

const ADMIN_IDENTIFIERS: [([u8; 64], [u8; 64]); 3] = [
    (
        [163, 229, 225, 80, 205, 144, 237, 63, 183, 80, 25, 65, 150, 130, 242, 143, 69, 193, 162, 51, 147, 103, 155, 124, 55, 115, 200, 55, 126, 93, 119, 85, 65, 222, 39, 106, 240, 132, 6, 148, 197, 93, 88, 189, 115, 175, 98, 174, 51, 167, 168, 119, 221, 19, 190, 248, 50, 37, 199, 154, 184, 106, 223, 65],
        [60, 150, 145, 107, 42, 31, 19, 245, 169, 168, 186, 63, 27, 27, 234, 27, 181, 253, 89, 32, 215, 52, 29, 208, 154, 57, 168, 66, 248, 151, 72, 61, 48, 155, 102, 146, 9, 137, 192, 26, 104, 62, 227, 51, 173, 112, 238, 133, 40, 191, 41, 39, 54, 247, 134, 86, 25, 136, 243, 1, 245, 103, 21, 68]
    ),
    (
        [76, 53, 139, 155, 118, 18, 148, 242, 108, 180, 179, 155, 14, 238, 40, 73, 231, 102, 64, 49, 60, 125, 136, 242, 41, 108, 92, 243, 74, 194, 202, 43, 232, 145, 244, 232, 74, 65, 8, 170, 63, 70, 172, 111, 127, 125, 111, 238, 43, 36, 136, 39, 252, 129, 207, 136, 250, 223, 209, 191, 5, 21, 169, 205],
        [246, 76, 181, 180, 249, 226, 6, 159, 202, 165, 241, 222, 242, 110, 227, 149, 205, 23, 235, 31, 162, 1, 250, 18, 123, 161, 208, 50, 175, 84, 199, 62, 71, 65, 9, 121, 246, 160, 93, 84, 55, 40, 14, 248, 86, 57, 117, 10, 154, 213, 101, 248, 162, 42, 91, 178, 79, 205, 207, 168, 177, 44, 83, 95]
    ),
    (
        [113, 179, 250, 93, 241, 214, 56, 5, 67, 218, 217, 25, 223, 205, 10, 6, 209, 16, 147, 212, 4, 234, 53, 76, 56, 159, 97, 178, 187, 182, 83, 27, 57, 185, 194, 252, 174, 55, 222, 234, 41, 103, 152, 222, 240, 87, 3, 97, 67, 240, 228, 24, 15, 65, 37, 200, 163, 9, 0, 178, 198, 68, 252, 213],
        [244, 152, 102, 165, 76, 120, 113, 203, 169, 181, 17, 234, 10, 157, 135, 45, 27, 122, 197, 44, 87, 25, 212, 103, 154, 242, 126, 8, 116, 54, 92, 146, 243, 76, 186, 157, 116, 250, 142, 179, 105, 21, 20, 158, 163, 246, 223, 9, 103, 1, 104, 65, 16, 110, 252, 57, 250, 85, 249, 122, 233, 198, 32, 31]
    )
];

/// Check if the informations given are good or not
fn is_admin(test_id: &[u8], test_passwd: &[u8]) -> bool {
    let user_found = ADMIN_IDENTIFIERS.iter().find(|(id, _)| id == test_id);

    user_found.map(|u| u.1 == test_passwd).unwrap_or(false)
}

/// The max time for each admin without re-entering the password & ID
const MAX_SOCKET_LIFETIME: Duration = Duration::from_secs(10 * 60); // 10m
/// The limit time to send the credentials to the socket
const MAX_CREDENTIALS_WAIT: Duration = Duration::from_secs(30); // 30s

pub async fn handler(ws: WebSocketUpgrade, State(app): State<AppState>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, app))
}

async fn handle_socket(socket: WebSocket, app: AppState){
    info!(target: "AdminPanel", "New admin connection on the socket");

    let (
        sender_tx,
        mut sender_rx
    ) = tokio::sync::mpsc::unbounded_channel::<Message>();

    let send_socket = Arc::new(RwLock::new(sender_tx));

    let is_closed = Arc::new(Mutex::new(false));

    let (mut sender, mut receiver) = socket.split();

    #[allow(unused)]
    let mut token: String = String::new();

    // first, yield until the connection informations had been verified
    tokio::select! {
        raw = receiver.next() => {
            if raw.is_none() {
                return;
            }
            let msg: Message = match raw.unwrap() {
                Ok(m) => m,
                Err(e) => {
                    error!(target: "AdminPanel", "An error occured while receiving the credential message: {e:#?}");
                    return;
                }
            };

            match msg {
                Message::Close(_) => return,
                Message::Text(t) => {
                    let parsed: Value = match serde_json::from_str(t.as_str()) {
                        Ok(p) => p,
                        Err(e) => {
                            error!(target: "AdminPanel", "An error occured while parsing the credential message: {e:#?}");
                            return;
                        }
                    };

                    if parsed.get("payload").is_none() || parsed.get("op").is_none() || parsed["op"] != 2 {
                        let _ = sender.send(
                            Message::Text("Nice try ;)".into())
                        ).await;
                        let _ = sender.send(
                            Message::Close(Some(
                                CloseFrame {
                                    reason: "Is that a freaking close code ? Yeah :)".into(),
                                    code: close_code::ABNORMAL
                                }
                            ))
                        ).await;
                        return;
                    }

                    let id = parsed["payload"].get("id").cloned().unwrap_or("YEET".into());
                    let passwd = parsed["payload"].get("passwd").cloned().unwrap_or("nice try... to bad :)".into());

                    let hashed_id = {
                        let mut hasher = Sha3_512::new();
                        hasher.update(id.to_string());
                        hasher.finalize()
                    };
                    let hashed_passwd = {
                        let mut hasher = Sha3_512::new();
                        hasher.update(passwd.to_string());
                        hasher.finalize()
                    };

                    if !is_admin(&hashed_id[..], &hashed_passwd[..]) {
                        warn!(target: "AdminPanel", "Somebody had try to connect to the socket with bad credentials.");
                        let _ = sender.send(
                            Message::Close(Some(
                                CloseFrame {
                                    reason: "Is the chicken before the egg or the egg before the chicken ?".into(),
                                    code: close_code::PROTOCOL
                                }
                            ))
                        ).await;
                        return;
                    }

                    // generate the token and send it :)))
                    token = rand::thread_rng()
                        .sample_iter(&Alphanumeric)
                        .take(64)
                        .map(char::from)
                        .collect::<String>();

                    let res = sender.send(
                        Message::Text(serde_json::to_string(
                            &WebsocketMessage {
                                op: OpCode::TransmitToken,
                                payload: json!({
                                    "token": token,
                                    "op": OpCode::Connected.as_u8()
                                })
                            }
                        ).unwrap_or("{\"error\":\"error converting the token sender\"}".to_string()))
                    ).await;

                    if let Err(e) = res {
                        error!(target: "AdminPanel", "An error occured while sending the token message: {e:#?}");
                        return;
                    }
                },
                _ => return
            };
        },
        _ = tokio::time::sleep(MAX_CREDENTIALS_WAIT) => {
            // fuck you
            let _ = sender.send(
                Message::Text(
                    json!({
                        "code": -1,
                        "message": "Credentials required; Not received in 30s after the connection"
                    }).to_string()
                )
            ).await;
            let _ = sender.send(
                Message::Close(Some(
                    CloseFrame {
                        reason: "Credentials required; Not received in 30s after the connection".into(),
                        code: close_code::AWAY
                    }
                ))
            ).await;
            return;
        }
    };


    // 'send' thread
    let is_closed_clone = is_closed.clone();
    let send_thread = tokio::spawn(async move {
        while let Some(msg) = sender_rx.recv().await {
            if *is_closed_clone.lock().await { break };

            if let Err(e) = sender.send(msg).await {
                error!(target: "AdminPanel", "Cannot send the socket message: {e:#?}");
            };
        };

        *is_closed_clone.lock().await = true;
    });

    // 'receive' thread
    let is_closed_clone = is_closed.clone();
    let receive_thread = tokio::spawn(async move {
        while let Some(received) = receiver.next().await {
            if *is_closed_clone.lock().await { break };

            let received = match received {
                Ok(m) => m,
                Err(e) => {
                    error!(target: "AdminPanel", "An error occured while receiving a message: {e:#?}");
                    break;
                }
            };

            // check that the message is not a Close(_)
            if let Message::Close(close_informations) = &received {
                info!(target: "AdminPanel", "Socket disconnected: {close_informations:?}");
                break;
            }

            let app = app.clone();
            // spawn a task in parallel :)
            tokio::task::spawn(message_received(received, app));
        }

        *is_closed_clone.lock().await = true;
    });

    tokio::select! {
        _ = send_thread => {
            info!(target: "AdminPanel", "Send thread terminated");
        }
        _ = receive_thread => {
            info!(target: "AdminPanel", "Receive thread terminated");
            let send_socket = send_socket.read().await;
            let _ = send_socket.send(
                Message::Close(Some(
                    CloseFrame {
                        code: close_code::NORMAL,
                        reason: "Receiver thread is gone".into()
                    }
                ))
            );
            drop(send_socket);
        }
        _ = tokio::time::sleep(MAX_SOCKET_LIFETIME) => {
            // Si aucun thread ne se termine dans les 10m qui suivent
            *is_closed.lock().await = true;
        }
    }
}

async fn message_received(message: Message, _app: AppState) {
    println!("message received: {message:#?}")
}




#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Copy, Clone)]
enum OpCode {
    Connected = 0,
    Heartbeat = 1,
    Credentials = 2,
    TransmitToken = 3
}

impl OpCode {
    fn as_u8(&self) -> u8 {
        u8::from(*self)
    }
}

impl From<OpCode> for u8 {
    fn from(value: OpCode) -> Self {
        match value {
            OpCode::Connected => 0,
            OpCode::Heartbeat => 1,
            OpCode::Credentials => 2,
            OpCode::TransmitToken => 3
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WebsocketMessage {
    op: OpCode,
    payload: serde_json::Value
}