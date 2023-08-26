use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use chrono::Local;
use log::error;
use regex::Regex;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use client::manager::cache::CacheManager;
use client::manager::shard::ShardManager;
use client::models::presence::Activity;
use client::typemap::Type;
use config::{Config, StatusConfig};


/// All possible states for a shard
#[allow(dead_code)]
pub enum ShardState {
    Normal,
    Dev,
    Maintenance,
    Unavailable
}

/// Contain useful informations about a shard
pub struct Shard {
    /// The state of the shard
    state: ShardState,
    /// A status specific to the shard, if it exists
    ///
    /// If None, the shard will use the status from the `ALTERNATE_STATUS` vector
    custom: Option<Activity>
}

impl Shard {
    fn new_empty() -> Self {
        Self {
            state: ShardState::Normal,
            custom: None
        }
    }
}

/// Manage the status of all shards
#[derive(Clone)]
#[allow(dead_code)]
pub struct ShardStatusManager {
    shards: Arc<RwLock<HashMap<u64, Shard>>>,
    alternate_pool: Arc<JoinHandle<()>>,
    config: Arc<RwLock<StatusConfig>>
}

impl ShardStatusManager {
    pub async fn new(
        shard_manager: Arc<RwLock<ShardManager>>,
        config: Arc<RwLock<Config>>,
        cache: Arc<RwLock<CacheManager>>
    ) -> Self {
        let status_config = config.read().await.status.clone();
        let shards = Arc::new(RwLock::new(HashMap::new()));
        Self {
            alternate_pool: Arc::new(
                Self::start_alternate_status_pool(shards.clone(), shard_manager, config.clone(), cache)
            ),
            config: Arc::new(RwLock::new(status_config)),
            shards
        }
    }

    fn start_alternate_status_pool(
        shards: Arc<RwLock<HashMap<u64, Shard>>>,
        shard_manager: Arc<RwLock<ShardManager>>,
        config: Arc<RwLock<Config>>,
        cache: Arc<RwLock<CacheManager>>
    ) -> JoinHandle<()> {
        tokio::spawn(async move {
            // we wait 5 seconds to let the shards the time to connect
            sleep(Duration::from_secs(5)).await;

            let mut alternate_index: u64 = 0;

            loop {
                {
                    let shard_manager = shard_manager.read().await;
                    let mut shards = shards.write().await;

                    for (id, shard) in shard_manager.get_shards().iter() {
                        // if this shard is gone, we skip it
                        if !*shard.run.lock().await {
                            continue
                        }

                        // if the shard isn't registered yet, we register it
                        if shards.get(id).is_none() {
                            shards.insert(*id, Shard::new_empty());
                        }
                        // now we can get the shard from the hashmap safely
                        let cache_data = shards.get(id).unwrap();

                        // if a custom status was set, we use it
                        if let Some(custom_status) = &cache_data.custom {
                            if let Err(e) = format_status(custom_status.clone(), cache.clone(), config.clone()).await.set_presence(shard).await {
                                error!(target: "ShardStatusManager", "Failed to set the custom status for shard {}: {:?}", id, e);
                                continue;
                            };
                        }

                        // else, we use the state to choose the best status
                        match cache_data.state {
                            ShardState::Normal => {
                                // we set the alternative status

                                let status = {
                                    let config = config.read().await;
                                    config.status.alternate[alternate_index as usize].clone()
                                };

                                if let Err(e) = format_status(status, cache.clone(), config.clone()).await.set_presence(shard).await {
                                    error!(target: "ShardStatusManager", "Failed to set the alternate status for shard {}: {:?}", id, e);
                                    continue;
                                };
                            }
                            ShardState::Dev => {
                                // we set the dev status
                                let status = {
                                    let config = config.read().await;
                                    config.status.dev.clone()
                                };

                                if let Err(e) = format_status(status, cache.clone(), config.clone()).await.set_presence(shard).await {
                                    error!(target: "ShardStatusManager", "Failed to set the dev status for shard {}: {:?}", id, e);
                                    continue;
                                };
                            }
                            ShardState::Maintenance => {
                                // we set the maintenance status
                                let status = {
                                    let config = config.read().await;
                                    config.status.maintenance.clone()
                                };

                                if let Err(e) = format_status(status, cache.clone(), config.clone()).await.set_presence(shard).await {
                                    error!(target: "ShardStatusManager", "Failed to set the maintenance status for shard {}: {:?}", id, e);
                                    continue;
                                };
                            },
                            ShardState::Unavailable => {
                                // we set the unavailable status
                                let status = {
                                    let config = config.read().await;
                                    config.status.unavailable.clone()
                                };

                                if let Err(e) = format_status(status, cache.clone(), config.clone()).await.set_presence(shard).await {
                                    error!(target: "ShardStatusManager", "Failed to set the unavailable status for shard {}: {:?}", id, e);
                                    continue;
                                };
                            }
                        }
                    }
                }

                // we increment the index
                {
                    alternate_index += 1;

                    let config = config.read().await;

                    // if the index is out of bounds, we reset it
                    if alternate_index >= config.status.alternate.len() as u64 {
                        alternate_index %= config.status.alternate.len() as u64;
                    }
                }

                let interval = {
                    let config = config.read().await;
                    config.status.interval
                };

                // we wait 5 minutes before updating the status again :)
                sleep(Duration::from_secs(interval)).await;
            }
        })
    }
}

impl Type for ShardStatusManager {
    type Value = Self;
}


async fn format_status(
    status: Activity,
    cache: Arc<RwLock<CacheManager>>,
    config: Arc<RwLock<Config>>
) -> Activity
{
    let regex = Regex::new(r#"(\{\w+\})"#).unwrap();

    // use the regex to catch everything between {}
    let captures = regex.captures_iter(&status.name);

    let mut name = status.name.clone();
    for capture in captures {
        let capture = capture.get(0).unwrap();
        let capture = capture.as_str();

        // we get the value from the cache
        let value = match capture {
            "{client}" => {

                let mut client_user = {
                    let cache = cache.read().await;
                    cache.get_client_user().cloned()
                };

                if client_user.is_none() {
                    let cache = cache.read().await;
                    client_user = cache.get_client_user().cloned();
                }

                // update cache ?
                {
                    if let Some(client_user) = &client_user {
                        let mut cache = cache.write().await;
                        cache.update_client_user(client_user);
                    }
                }

                if let Some(client_user) = client_user {
                    client_user.username.clone()
                } else {
                    "Unknown".to_string()
                }
            },
            "{guilds}" => {
                let cache = cache.read().await;
                cache.get_guild_size().to_string()
            },
            "{users}" => {
                let cache = cache.read().await;
                cache.get_user_size().to_string()
            },
            "{channels}" => {
                let cache = cache.read().await;
                cache.get_channel_size().to_string()
            },
            "{version}" => {
                let config = config.read().await;
                config.version.clone()
            },
            "{day}" => {
                // format a date to day/month/year
                let now = Local::now();
                now.format("%d/%m/%Y").to_string()
            },
            _ => capture.to_string()
        };

        // we replace the capture with the value
        name = name.replace(capture, &value);
    };

    Activity {
        name,
        activity_type: status.activity_type.clone(),
        url: status.url.clone(),
        presence: status.presence.clone(),
    }
}