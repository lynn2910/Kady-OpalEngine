use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use futures_channel::mpsc::{UnboundedReceiver, UnboundedSender};
use futures_util::SinkExt;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use url::Url;
use futures_util::stream::{FusedStream, SplitSink, SplitStream, StreamExt};
use log::{error, info, trace, warn};
use serde_json::{json, Value};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tokio_tungstenite::tungstenite::error::ProtocolError;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
#[allow(unused_imports)] // They are used in the 'json!' macro
use crate::constants::{ GATEWAY_URL, BROWSER, DEVICE };
use error::{ Result, Error, GatewayError };
use crate::models::presence::Presence;

#[derive(Debug)]
pub enum ShardState {
    Connecting,
    Connected,
    Disconnected
}

#[derive(Debug)]
#[allow(dead_code)]
pub(crate) enum GatewayEvent {
    Event(u32),
    Connected,
    Reconnected,
}

impl From<u64> for GatewayEvent {
    fn from(value: u64) -> Self {
        match value {
            0 => GatewayEvent::Event(0),
            1 => GatewayEvent::Event(1),
            7 => GatewayEvent::Event(7),
            9 => GatewayEvent::Event(9),
            10 => GatewayEvent::Event(10),
            11 => GatewayEvent::Event(11),
            _ => panic!("Unknown gateway event")
        }
    }
}

#[derive(Debug)]
/// Represent a shard
pub struct Shard {
    pub(crate) threads: ShardThreads,
    pub(crate) sending_channel: UnboundedSender<Message>,
    pub(crate) connected: Arc<Mutex<bool>>,
    pub        run: Arc<Mutex<bool>>,
    pub        id: u64,
    pub        total: u64,
    pub        intents: u64,
    pub        state: ShardState,
    pub        ping: Arc<RwLock<u128>>,
}

pub(crate) struct ShardChannels {
    pub(crate) shard: Shard,
    /// Channel that contain the messages received from the websocket
    pub(crate) received: UnboundedReceiver<Value>,
}

#[derive(Debug)]
pub(crate) struct ShardThreads {
    pub(crate) heartbeat: JoinHandle<()>,
    pub(crate) sending: JoinHandle<()>,
    pub(crate) received: JoinHandle<()>
}

impl Shard {
    pub(crate) async fn connect(
        id: u64,
        total: u64,
        token: String,
        intents: u64
    ) -> Result<ShardChannels>
    {
        let url = Url::parse(GATEWAY_URL).unwrap();

        let (ws_stream, _) = match connect_async(url).await {
            Ok(d) => d,
            Err(err) => return Err(Error::Gateway(GatewayError::ShardConnectionError(err.to_string())))
        };

        let is_shard_connected = Arc::new(Mutex::new(true));

        let (received_tx,     received_rx) = futures_channel::mpsc::unbounded::<Value>();
        let ( sending_tx, mut sending_rx) = futures_channel::mpsc::unbounded::<Message>();

        // Send payload
        let (mut write, mut read) = ws_stream.split();

        // wait for the "hello" message
        let heartbeat_interval: u64 = {
            let msg = if let Some(m) = Self::read_hello(&mut read).await { m } else {
                return Err(Error::Gateway(GatewayError::ShardMessageError("Failed to read hello message".to_string())))
            };

            let json: Value = match serde_json::from_str(msg.as_str()) {
                Ok(d) => d,
                Err(err) => return Err(Error::Gateway(GatewayError::ParsingError(format!("Failed to parse hello message: {:?}", err))))
            };

            if !json.is_object() {
                return Err(Error::Gateway(GatewayError::ParsingError("Hello message is not an object".to_string())))
            }

            let op = match json["op"].as_u64() {
                Some(d) => d,
                None => return Err(Error::Gateway(GatewayError::ParsingError("Hello message has no 'op' field".to_string())))
            };

            if op != 10 {
                return Err(Error::Gateway(GatewayError::ParsingError(format!("Hello message has wrong op code: {}", op))))
            }

            match json["d"]["heartbeat_interval"].as_u64() {
                Some(d) => d,
                None => return Err(Error::Gateway(GatewayError::ParsingError("Hello message has no 'heartbeat_interval' field".to_string())))
            }
        };

        // send payload
        Self::send_payload(&mut write, token, intents).await?;

        // configure the handshake system
        let last_heartbeat = Arc::new(Mutex::new(std::time::Instant::now()));
        let ping = Arc::new(RwLock::new(0u128));
        let run_shard = Arc::new(Mutex::new(true));

        // spawn thread heartbeats
        let last_heartbeat_clone = last_heartbeat.clone();
        let run_shard_clone = run_shard.clone();
        let sending_tx_clone = sending_tx.clone();
        let heartbeat_thread = tokio::spawn(async move {
            'heartbeat: loop {
                // if we want to stop the heartbeat system, we simply check this
                if !*run_shard_clone.lock().await {
                    break 'heartbeat;
                }

                let mut last_heartbeat = last_heartbeat_clone.lock().await;
                // send heartbeat
                let msg = json!({
                    "op": 1,
                    "d": last_heartbeat.elapsed().as_millis()
                });
                // update last heartbeat
                *last_heartbeat = std::time::Instant::now();

                // free the lock
                drop(last_heartbeat);

                // request a message to be sent
                if let Err(e) = sending_tx_clone.unbounded_send(Message::Text(msg.to_string())) {
                    error!(target: "HeartbeatShard", "Error while sending heartbeat: {:?}", e);
                    continue
                }

                // wait for heartbeat interval
                sleep(Duration::from_millis(heartbeat_interval)).await;
            }
        });

        // Spawn thread to send messages
        let run_shard_clone = run_shard.clone();
        let is_shard_connected_clone = is_shard_connected.clone();
        let sending_thread = tokio::spawn(async move {
            'sending: loop {
                // if we want to stop the heartbeat system, we simply check this
                if !*run_shard_clone.lock().await {
                    break 'sending;
                }

                if sending_rx.is_terminated() {
                    warn!(target: "SendingShard", "Sending thread for the shard {id} is closing");
                    {
                        let mut is_shard_connected = is_shard_connected_clone.lock().await;
                        *is_shard_connected = false;
                    }
                    break 'sending;
                }

                let msg = sending_rx.next().await;
                if msg.is_none() {
                    warn!(target: "SendingShard", "Sending thread for the shard {id} is closing");
                    continue
                }

                let msg = msg.unwrap();

                if let Err(e) = write.send(msg).await {
                    error!(target: "SendingShard", "Error while sending message to websocket: {e:?}");
                    continue;
                }
            }
        });

        // Spawn thread to read messages
        let last_heartbeat_clone = last_heartbeat.clone();
        let run_shard_clone = run_shard.clone();
        let is_shard_connected_clone = is_shard_connected.clone();
        let ping_clone = ping.clone();
        let received_thread = tokio::spawn(async move {
            'receive: loop {
                // if we want to stop the heartbeat system, we simply check this
                if !*run_shard_clone.lock().await {
                    break 'receive;
                }

                let msg = match read.next().await {
                    Some(msg) => {
                        match msg {
                            Ok(d) => d,
                            Err(err) => {
                                match err {
                                    // protocol error
                                    tokio_tungstenite::tungstenite::Error::Protocol(protocol) => {
                                        match protocol {
                                            ProtocolError::ResetWithoutClosingHandshake => {
                                                *is_shard_connected_clone.lock().await = false;
                                                break 'receive;
                                            },
                                            protocol => {
                                                *is_shard_connected_clone.lock().await = false;
                                                error!(target: "ReceivingShard", "A protocol error occured: {protocol:?}");
                                                break 'receive;
                                            }
                                        }
                                    },
                                    tokio_tungstenite::tungstenite::Error::AlreadyClosed | tokio_tungstenite::tungstenite::Error::ConnectionClosed => {
                                        let mut is_shard_connected = is_shard_connected_clone.lock().await;
                                        if *is_shard_connected {
                                            *is_shard_connected = false
                                        };
                                        break 'receive;
                                    },
                                    tokio_tungstenite::tungstenite::Error::Tls(tls_error) => {
                                        error!(target: "ReceivingShard", "A TLS error occured: {tls_error:#?}");

                                        // TODO write logs

                                        *is_shard_connected_clone.lock().await = false;
                                        break 'receive;
                                    }
                                    _ => {
                                        error!(target: "ReceivingShard", "Error while reading message: {:?}", err);
                                        *is_shard_connected_clone.lock().await = false;
                                        break 'receive;
                                    }
                                }
                            }
                        }
                    },
                    None => {
                        #[cfg(feature = "debug")]
                        info!(target: "ReceivingShard", "Websocket for shard {id} is closed", id = id);
                        *is_shard_connected_clone.lock().await = false;
                        break 'receive;
                    }
                };

                if let Message::Close(close_frame) = &msg {
                    // a close frame was received, we need to close the connection
                    *is_shard_connected_clone.lock().await = false;

                    let code = close_frame.as_ref().map(|cf| cf.code);

                    let reason = if let Some(cf) = close_frame {
                        cf.reason.to_string()
                    } else {
                        String::new()
                    };


                    #[cfg(feature = "debug")]
                    warn!(target: "ReceivingShard", "Websocket for shard {id} is closing\n    code: {code:?}\n    reason: {reason:?}", id = id);

                    break 'receive;
                }

                // if we want to stop the heartbeat system, we simply check this
                if !*run_shard_clone.lock().await {
                    break 'receive;
                }

                // message received! Parse to JSON and send to channel
                let content: Value = match serde_json::from_str(msg.to_text().unwrap()) {
                    Ok(d) => d,
                    #[cfg(not(feature = "debug"))]
                    #[allow(unreachable_patterns)]
                    Err(_) => continue,
                    #[cfg(feature = "debug")]
                    #[allow(unreachable_patterns)]
                    Err(err) => {
                        error!(target: "ReceivingShard", "Error while parsing message: {:?}", err);
                        continue
                    }
                };

                if let Some(op) = content["op"].as_u64() {
                    if op == 11 {
                        #[cfg(feature = "debug")]
                        trace!(target: "ReceivingShard", "Heartbeat ACK received for shard {id}: {content:?}");

                        // Heartbeat ACK
                        let mut last_heartbeat = last_heartbeat_clone.lock().await;
                        let mut ping = ping_clone.write().await;
                        *ping = last_heartbeat.elapsed().as_millis();
                        *last_heartbeat = std::time::Instant::now();

                        drop(last_heartbeat);
                        drop(ping);
                        continue
                    }
                }

                if received_tx.is_closed() {
                    {
                        let mut is_shard_connected = is_shard_connected_clone.lock().await;
                        *is_shard_connected = false;
                    }
                    break 'receive;
                }

                // Send to channel
                match received_tx.unbounded_send(content) {
                    Ok(_) => (),
                    Err(err) => {
                        error!(target: "ReceivingShard", "Error while sending message to channel: {:?}", err);
                        {
                            let mut is_shard_connected = is_shard_connected_clone.lock().await;
                            *is_shard_connected = false;
                        }
                        break 'receive;
                    }
                }
            };
        });

        Ok(ShardChannels {
            shard: Self {
                id,
                total,
                intents,
                ping,
                state: ShardState::Connecting,
                sending_channel: sending_tx,
                run: run_shard.clone(),
                connected: is_shard_connected,
                threads: ShardThreads {
                    heartbeat: heartbeat_thread,
                    sending: sending_thread,
                    received: received_thread
                }
            },
            received: received_rx,
        })
    }

    async fn send_payload(
        write: &mut SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
        token: String,
        intents: u64
    )-> Result<()>
    {
        let payload = json!({
            "op": 2,
            "d": {
                "token": token,
                "intents": intents,
                "properties": {
                    "$os": std::env::consts::OS,
                    "$browser": BROWSER,
                    "$device": DEVICE
                }
            }
        });

        if let Err(e) = write.send(Message::Text(payload.to_string())).await {
            return Err(Error::Gateway(GatewayError::PayloadError(format!("Failed to send payload: {:?}", e))))
        };

        Ok(())
    }

    async fn read_hello(read: &mut SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>) -> Option<String> {
        let msg = match read.next().await {
            Some(msg) => {
                match msg {
                    Ok(d) => d,
                    Err(err) => {
                        error!(target: "ShardHello", "Error while reading message: {:?}", err);
                        return None
                    }
                }
            },
            None => return None
        };

        // message received! Parse to JSON and send to channel
        let content = match msg.to_text() {
            Ok(d) => d,
            Err(err) => {
                error!(target: "ShardHello", "Error while parsing message: {:?}", err);
                return None
            }
        };

        Some(content.to_string())
    }

    pub async fn close(&mut self) -> Result<()> {
        *self.run.lock().await = false;

        if !self.sending_channel.is_closed() {
            match self.sending_channel.unbounded_send(Message::Close(Some(CloseFrame { code: CloseCode::Normal, reason: "Closing shard".into() }))) {
                Ok(_) => {
                    *self.connected.lock().await = false;
                    Ok(())
                },
                Err(e) => Err(Error::Gateway(GatewayError::InternChannelError(e.to_string())))
            }
        } else {
            Ok(())
        }
    }

    pub async fn set_presence(&self, presence: Presence) -> Result<()> {
        let payload = json!({
            "op": 3,
            "d": presence.to_json()
        });

        match self.sending_channel.unbounded_send(Message::Text(payload.to_string())) {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::Gateway(GatewayError::InternChannelError(e.to_string())))
        }
    }
}

#[derive(Debug)]
pub struct ShardManager {
    pub(crate) shards: HashMap<u64, Shard>
}

impl ShardManager {
    pub(crate) fn new() -> Self {
        Self {
            shards: HashMap::new()
        }
    }

    pub fn get_shards(&self) -> &HashMap<u64, Shard> {
        &self.shards
    }

    pub fn get_shard(&self, id: &u64) -> Option<&Shard> {
        self.shards.get(id)
    }

    pub fn get_mut_shard(&mut self, id: &u64) -> Option<&mut Shard> {
        self.shards.get_mut(id)
    }

    pub async fn close_shard(&mut self, id: &u64) -> Result<()> {
        if let Some(shard) = self.shards.get_mut(id) {
            shard.close().await
        } else {
            Err(Error::Gateway(GatewayError::ShardNotFound(id.to_string())))
        }
    }

    pub async fn close_all(&mut self) -> Result<()> {
        for (id, shard) in self.shards.iter_mut() {
            if let Err(e) = shard.close().await {
                error!(target: "ShardManager", "An error occured while closing the shard {id}: {e:?}");
            } else {
                info!(target: "ShardManager", "Shard {} closed", id);
            }
        }

        Ok(())
    }
}