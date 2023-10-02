use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use chrono::Utc;
use futures_util::StreamExt;
use log::{debug, error, info, warn};
use reqwest::header;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{ Serialize, Deserialize };
use serde_json::Value;
use tokio::sync::RwLock;
use error::{ Result, ApiError, Error };
use crate::constants::API_URL;
use crate::manager::cache::CacheManager;
use crate::manager::events::{Context, EventHandler};
use crate::manager::http::{HttpConfiguration, HttpManager};
use crate::manager::shard::{GatewayEvent, Shard, ShardChannels, ShardManager};
use crate::models::events::{GuildCreate, GuildDelete, GuildMemberAdd, GuildMemberUpdate, InteractionCreate, MessageCreate, MessageDelete, Ready};
use crate::models::guild::GuildMember;
use crate::models::interaction::Interaction;
use crate::models::message::Message;
use crate::typemap::{Type, TypeMap};

pub mod manager;
pub mod models;
mod utils;
pub mod constants;
pub mod typemap;

/// Contains session limit infos from the gateway
///
/// Is used to manage the websockets
///
/// See [here](https://discord.com/developers/docs/topics/gateway#get-gateway-bot) for more infos
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionLimit {
    /// Total number of identifies before getting rate-limited
    pub total: u32,
    /// Number of identifies remaining before getting rate-limited
    pub remaining: u32,
    /// Number of milliseconds before the limit resets
    pub reset_after: u32,
    /// Number of identify requests that can be made per 5 seconds
    pub max_concurrency: u32,
}

/// Represent the Discord Client
pub struct Client {
    //  Contains all shards
    // shards: HashMap<u64, Shard>,
    /// Store the bot token
    token: String,
    /// Store the number of shards
    shards_count: u64,
    /// Store the reqwest client
    pub http_manager: Arc<HttpManager>,
    /// Store the shard system
    pub shard_manager: Arc<RwLock<ShardManager>>,
    /// Store the gateway url
    gateway_url: String,
    /// Store the session limit
    session_limit: SessionLimit,
    /// Store the event manager
    events: Option<Arc<dyn EventHandler>>,
    /// Contains the cache
    pub cache: Arc<RwLock<CacheManager>>,
    /// Contains datas
    pub data: Arc<RwLock<TypeMap>>,
}

impl Client {
    /// Create a new client
    ///
    /// Automatically get gateway infos
    pub async fn new(token: String, http_configuration: HttpConfiguration) -> Self {
        // Build reqwest client
        let client = {
            let mut headers = HeaderMap::new();
            headers.insert(header::USER_AGENT, HeaderValue::from_str(constants::USER_AGENT).unwrap());
            headers.insert(header::AUTHORIZATION, HeaderValue::from_str(format!("Bot {token}").as_str()).unwrap());
            headers.insert(header::ACCEPT, HeaderValue::from_static("application/json"));

            reqwest::Client::builder()
                .default_headers(headers)
                .connect_timeout(http_configuration.connect_timeout)
                .build()
                .expect("Failed to build reqwest client")
        };

        // Get gateway infos
        let gateways_infos = match Self::get_gateway_infos(&client).await {
            Ok(d) => d,
            Err(err) => panic!("Failed to get gateway infos: {:?}", err)
        };

        let http_manager = HttpManager::new(http_configuration, client);

        http_manager.start_loop();

        // Return client
        Self {
            token,
            shard_manager: Arc::new(RwLock::new(ShardManager::new())),
            http_manager: Arc::new(http_manager),
            events: None,
            cache: Arc::new(RwLock::new(CacheManager::new())),
            gateway_url: gateways_infos["url"]
                .as_str()
                .expect("Failed to get gateway url: No 'url' field")
                .to_string(),
            shards_count: gateways_infos["shards"]
                .as_u64()
                .expect("Failed to get shards count: No 'shards' field"),
            session_limit: serde_json::from_value(gateways_infos["session_start_limit"].clone())
                .expect("Failed to parse session limit"),
            data: Arc::new(RwLock::new(TypeMap::new()))
        }
    }

    pub async fn get_gateway_infos(client: &reqwest::Client) -> Result<Value> {
        let res = match client.get(format!("{}/gateway/bot", API_URL)).send().await {
            Ok(d) => d,
            Err(err) => return Err(Error::Api(ApiError::RequestError(err.to_string())))
        };

        match res.json().await {
            Ok(d) => Ok(d),
            Err(err) => Err(Error::Api(ApiError::InvalidJson(err.to_string())))
        }
    }

    /// Connect the client using the token and intents
    ///
    /// Will spawn two threads per shard
    ///
    /// This function is non-blocking
    pub async fn start(self, intents: u64) -> Result<()> {
        if self.events.is_none() {
            panic!("No event handler set");
        }
        let shards_count = self.shards_count;

        {
            // update application in the cache
            let application = self.http_manager.client.fetch_application().await.expect("Failed to fetch application");
            let client_user = self.http_manager.client.fetch_client_user().await.expect("Failed to fetch client user");

            let mut cache = self.cache.write().await;
            match application {
                Ok(application) => cache.update_application(&application),
                Err(err) => error!("Failed to fetch application: {:?}", err)
            };
            match client_user {
                Ok(client_user) => cache.update_client_user(&client_user),
                Err(err) => error!("Failed to fetch client user: {:?}", err)
            };
        }

        let arc_client = Arc::new(tokio::sync::Mutex::new(self));

        let shards_already_spawned: Arc<tokio::sync::Mutex<Vec<u64>>> = Arc::new(tokio::sync::Mutex::new(Vec::new()));

        let mut first_loop = true;

        'shards: loop {
            let mut shards_fully_disconnected = 0;
            for i in 0..shards_count {
                let client = arc_client.clone();
                let client = client.lock().await;

                // check if shard is already connected
                let (running, is_connected) = {
                    let shard_manager = client.shard_manager.read().await;
                    let shard = shard_manager.shards.get(&i);

                    // if the shard was already spawned and is fully disconnected, we skip it
                    if let Some(shard) = &shard {
                        let shards_already_spawned = shards_already_spawned.lock().await;

                        let running = *shard.run.lock().await;
                        let already_spawned = shards_already_spawned.contains(&i);

                        if !running && already_spawned {
                            shards_fully_disconnected += 1;
                            continue;
                        }
                    }

                    match shard {
                        None => (true, false),
                        Some(shard) => (*shard.run.lock().await, *shard.connected.lock().await)
                    }
                };

                // if shard is already connected, skip it
                if running && !is_connected {
                    #[cfg(feature = "debug")]
                    debug!(target: "ShardHandler", "Connecting shard {}", i);

                    // tell that the shard is already spawned so it won't be spawned again if the shard is closed
                    {
                        let mut shards_already_spawned = shards_already_spawned.lock().await;
                        if !shards_already_spawned.contains(&i) {
                            shards_already_spawned.push(i);
                        }
                    }

                    // remove shard if it exists
                    {
                        let mut shard_manager = client.shard_manager.write().await;
                        if let Some(old_shard) = shard_manager.shards.get(&i) {

                            #[cfg(feature = "debug")]
                            info!(target: "ShardHandler", "Clearing old shard {}", i);

                            // clear threads
                            old_shard.threads.heartbeat.abort_handle().abort();
                            old_shard.threads.received.abort_handle().abort();
                            old_shard.threads.sending.abort_handle().abort();

                            // clear channels
                            old_shard.sending_channel.close_channel();
                        }

                        shard_manager.shards.remove(&i);
                    }

                    // init shard
                    let ShardChannels {
                        shard,
                        mut received
                    } = Shard::connect(
                        i,
                        client.shards_count,
                        client.token.clone(),
                        intents
                    ).await?;

                    // store shard
                    {
                        let mut shard_manager = client.shard_manager.write().await;
                        shard_manager.shards.insert(i, shard);
                    }

                    let client_arc = arc_client.clone();
                    tokio::spawn(async move {
                        // loop {
                        //     // we will manage each event received
                        //
                        //     // get the message from the channel
                        //     let data: Value = match received.next().await {
                        //         Some(d) => d,
                        //         None =>  break
                        //     };
                        //
                        //     // match operators to get the event type
                        //     let op = match data["op"].as_u64() {
                        //         Some(d) => d,
                        //         None => continue
                        //     };
                        //
                        //     {
                        //         let mut client = client_arc.lock().await;
                        //         client.event_triggered(op.into(), data, i);
                        //     }
                        // }

                        while let Some(data) = received.next().await {
                            // match operators to get the event type
                            let op = match data["op"].as_u64() {
                                Some(d) => d,
                                None => continue
                            };

                            {
                                let mut client = client_arc.lock().await;
                                client.event_triggered(op.into(), data, i);
                            }
                        }
                    });
                }

                // to avoid ressources competition, we drop the client
                drop(client);
            };

            // if all shards are fully disconnected, we stop the loop
            if shards_fully_disconnected == shards_count {
                break 'shards;
            }

            if first_loop {
                let arc_client = arc_client.lock().await;

                if let Some(ref events) = arc_client.events {
                    let ctx = Context::new(
                        arc_client.data.clone(),
                        0,
                        arc_client.http_manager.client.clone(),
                        arc_client.shard_manager.clone(),
                        arc_client.cache.clone()
                    );

                    let events_clone = events.clone();
                    tokio::spawn(async move {
                        let _ = events_clone.start(ctx).await;
                    });
                }

                drop(arc_client);

                first_loop = false;
            }

            // wait 1s to don't burn the CPU
            tokio::time::sleep(core::time::Duration::from_millis(500)).await;
        }

        info!(target: "Client", "The client is fully closed");

        Ok(())
    }

    /// Register the event handler
    pub fn event_handler<H: EventHandler + 'static>(&mut self, handler: H) -> &mut Self {
        info!(target: "Client", "Event handler registered");
        self.events = Some(Arc::new(handler));
        self
    }




    fn event_triggered(&mut self, op: GatewayEvent, content: Value, shard: u64) {
        match op {
            GatewayEvent::Event(_) => self.gateway_event(op, content, shard),
            _ => {
                #[cfg(feature = "debug")]
                warn!("Unhandled gateway op received: {:?}", op);
            }
        }
    }

    fn gateway_event(&mut self, _op: GatewayEvent, mut content: Value, shard: u64) {
        let event = match content["t"].as_str() {
            Some(d) => d,
            None => {
                warn!("No event name received");
                return;
            }
        };

        match event {
            "READY" => {
                let ready: Ready = Ready {
                    timestamp: Utc::now(),
                    shard
                };

                if let Some(events) = self.events.as_ref() {
                    let ctx = Context::new(
                        self.data.clone(),
                        ready.shard,
                        self.http_manager.client.clone(),
                        self.shard_manager.clone(),
                        self.cache.clone()
                    );

                    let events_clone = events.clone();
                    tokio::spawn(async move {
                        let _ = events_clone.ready(ctx, ready).await;
                    });
                }
            },
            "GUILD_CREATE" => {
                content["d"]["shard"] = Value::from(shard);
                let guild_create: GuildCreate = match serde_path_to_error::deserialize(content["d"].clone()) {
                    Ok(d) => d,
                    Err(err) => {
                        #[cfg(feature = "debug")]
                        warn!("Failed to parse guild create event: {:#?}", err);
                        return;
                    }
                };

                if let Some(events) = self.events.as_ref() {
                    let ctx = Context::new(
                        self.data.clone(),
                        guild_create.shard,
                        self.http_manager.client.clone(),
                        self.shard_manager.clone(),
                        self.cache.clone()
                    );

                    let events_clone = events.clone();
                    tokio::spawn(async move {
                        let _ = events_clone.guild_create(ctx, guild_create).await;
                    });
                }
            },
            "GUILD_DELETE" => {
                content["d"]["shard"] = Value::from(shard);
                let guild_delete: GuildDelete = match serde_path_to_error::deserialize(content["d"].clone()) {
                    Ok(d) => d,
                    Err(err) => {
                        #[cfg(feature = "debug")]
                        warn!("Failed to parse guild delete event: {:#?}", err);
                        return;
                    }
                };

                if let Some(events) = self.events.as_ref() {
                    let ctx = Context::new(
                        self.data.clone(),
                        guild_delete.shard,
                        self.http_manager.client.clone(),
                        self.shard_manager.clone(),
                        self.cache.clone()
                    );

                    let events_clone = events.clone();
                    tokio::spawn(async move {
                        let _ = events_clone.guild_delete(ctx, guild_delete).await;
                    });
                }
            },
            "MESSAGE_CREATE" => {
                let parsed_message: Message = match serde_path_to_error::deserialize(content["d"].clone()) {
                    Ok(d) => d,
                    Err(err) => {
                        #[cfg(feature = "debug")]
                        warn!("Failed to parse message create event: {:#?}", err);
                        return;
                    }
                };

                let message_create = MessageCreate {
                    message: parsed_message,
                    guild_id: content.get("guild_id").map(|i| i.to_string().into()),
                    shard
                };

                if let Some(events) = self.events.as_ref() {
                    let ctx = Context::new(
                        self.data.clone(),
                        message_create.shard,
                        self.http_manager.client.clone(),
                        self.shard_manager.clone(),
                        self.cache.clone()
                    );

                    let events_clone = events.clone();
                    tokio::spawn(async move {
                        let _ = events_clone.message_create(ctx, message_create).await;
                    });
                }
            },
            "MESSAGE_DELETE" => {
                content["d"]["shard"] = Value::from(shard);
                let message_delete: MessageDelete = match serde_json::from_value(content["d"].clone()) {
                    Ok(d) => d,
                    Err(err) => {
                        #[cfg(feature = "debug")]
                        warn!("Failed to parse message delete event: {:?}", err);
                        return;
                    }
                };

                if let Some(events) = self.events.as_ref() {
                    let ctx = Context::new(
                        self.data.clone(),
                        message_delete.shard,
                        self.http_manager.client.clone(),
                        self.shard_manager.clone(),
                        self.cache.clone()
                    );

                    let events_clone = events.clone();
                    tokio::spawn(async move {
                        let _ = events_clone.message_delete(ctx, message_delete).await;
                    });
                }
            },
            "GUILD_MEMBER_ADD" => {
                content["d"]["shard"] = Value::from(shard);
                let guild_member: GuildMember = match serde_path_to_error::deserialize(content["d"].clone()) {
                    Ok(d) => d,
                    Err(err) => {
                        warn!("Failed to parse guild member add event: {:#?}", err);
                        return;
                    }
                };

                let guild_member_add= GuildMemberAdd {
                    member: guild_member,
                    guild_id: content["d"]["guild_id"].to_string().into(),
                    shard,
                };

                if let Some(events) = self.events.as_ref() {
                    let ctx = Context::new(
                        self.data.clone(),
                        guild_member_add.shard,
                        self.http_manager.client.clone(),
                        self.shard_manager.clone(),
                        self.cache.clone()
                    );

                    let events_clone = events.clone();
                    tokio::spawn(async move {
                        let _ = events_clone.guild_member_add(ctx, guild_member_add).await;
                    });
                }
            },
            "GUILD_MEMBER_UPDATE" => {
                content["d"]["shard"] = Value::from(shard);
                let guild_member: GuildMember = match serde_path_to_error::deserialize(content["d"].clone()) {
                    Ok(d) => d,
                    Err(err) => {
                        #[cfg(feature = "debug")]
                        warn!("Failed to parse guild member update event: {:#?}", err);
                        return;
                    }
                };

                let guild_member_update = GuildMemberUpdate {
                    member: guild_member,
                    shard,
                };

                if let Some(events) = self.events.as_ref() {
                    let ctx = Context::new(
                        self.data.clone(),
                        guild_member_update.shard,
                        self.http_manager.client.clone(),
                        self.shard_manager.clone(),
                        self.cache.clone()
                    );

                    let events_clone = events.clone();
                    tokio::spawn(async move {
                        let _ = events_clone.guild_member_update(ctx, guild_member_update).await;
                    });
                }
            },
            "INTERACTION_CREATE" => {
                let interaction: Interaction = match serde_path_to_error::deserialize(content["d"].clone()) {
                    Ok(d) => d,
                    Err(err) => {
                        #[cfg(feature = "debug")]
                        warn!("Failed to parse interaction create event: {:#?}", err);
                        return;
                    }
                };

                let interaction_create = InteractionCreate { interaction, shard };

                if let Some(events) = self.events.as_ref() {
                    let ctx = Context::new(
                        self.data.clone(),
                        interaction_create.shard,
                        self.http_manager.client.clone(),
                        self.shard_manager.clone(),
                        self.cache.clone()
                    );

                    let events_clone = events.clone();
                    tokio::spawn(async move {
                        let _ = events_clone.interaction_create(ctx, interaction_create).await;
                    });
                }
            }
            _ => {
                #[cfg(feature = "debug")]
                warn!("Unhandled gateway event received: {:?}", event);
            }
        }
    }


    /// Insert a value inside the client, to use INSIDE the events
    pub async fn insert_data<T: Type>(&self, value: T) -> Option<Box<T>> {
        let mut docker = self.data.write().await;
        docker.insert::<T>(value)
    }

    /// Remove a element from the data container
    pub async fn remove_data<T: Type>(&self) -> Option<Box<T>> {
        let mut docker = self.data.write().await;
        docker.remove::<T>()
    }

    /// Access a element from the data, and return a cloned, if present
    pub async fn get_data<T: Type>(&self) -> Option<T> {
        let docker = self.data.read().await;
        docker.get::<T>().cloned()
    }
}

impl Debug for Client {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("shard_manager", &self.shard_manager)
            .field("token", &"private_field")
            .field("shards_count", &self.shards_count)
            .field("client", &"private_field")
            .field("gateway_url", &self.gateway_url)
            .field("session_limit", &self.session_limit)
            .finish()
    }
}