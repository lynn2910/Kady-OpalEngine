//! This module is used to manage the SIGINT, SIGTERM, and SIGQUIT signals.

use std::collections::HashMap;
use std::fs;
use std::process::exit;
use std::sync::Arc;
use chrono::Utc;
#[allow(unused_imports)]
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
#[cfg(unix)]
use signal_hook::consts::{SIGINT, SIGTERM};
#[cfg(unix)]
use signal_hook::iterator::exfiltrator::SignalOnly;
#[cfg(unix)]
use signal_hook::iterator::SignalsInfo;
use tokio::sync::RwLock;
use client::manager::http::HttpManager;
use client::manager::shard::ShardManager;
use config::Config;

#[cfg(unix)]
pub(crate) fn spawn_manager(
    http_manager: Arc<HttpManager>,
    shard_manager: Arc<RwLock<ShardManager>>
)
{
    // register all signals
    let sigs = vec![SIGINT, SIGTERM];

    let mut signals = SignalsInfo::<SignalOnly>::new(&sigs).expect("Failed to register signals");

    tokio::spawn(async move {
        for info in &mut signals {
            match info {
                SIGINT => {
                    println!();
                    info!(target: "Signal", "Received SIGINT");
                    stop(http_manager.clone(), shard_manager.clone()).await
                },
                SIGTERM => {
                    println!();
                    info!(target: "Signal", "Received SIGTERM");
                    stop(http_manager.clone(), shard_manager.clone()).await
                }
                unhandled => {
                    warn!(target: "Signal", "Unhandled signal {:?}", unhandled);
                }
            }
        }
    });
}

pub(crate) async fn stop(
    http_manager: Arc<HttpManager>,
    shard_manager: Arc<RwLock<ShardManager>>
)
{
    info!(target: "Core", "Stopping the client...");
    {
        let mut shard_manager = shard_manager.write().await;
        if let Err(e) = shard_manager.close_all().await {
            error!(target: "Core", "Failed to close all shards: {:?}", e);
            exit(1)
        };
    }

    info!(target: "Core", "Stopping the HTTP client...");
    http_manager.stop().await;

    info!(target: "Core", "Exiting...");
    exit(0)
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct MemoryReport {
    cache: CacheMemoryReport,
    shards: ShardMemoryReport,
    unit: String,
    total: f64,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct CacheMemoryReport {
    pub channels: f64,
    pub guilds: f64,
    pub users: f64,
    pub application: f64,
    pub client_user: f64,
    pub cache_total: f64
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct ShardMemoryReport {
    pub shards: HashMap<u64, f64>,
    pub total: f64
}

/// Create a memory report
///
/// Paramater 'absolute_values" is used to convert to Kb if false.
pub(crate) async fn report_memory_usage(
    cache_lock: Arc<RwLock<client::manager::cache::CacheManager>>,
    shard_manager: Arc<RwLock<ShardManager>>,
    config: Config,
    absolute_values: bool
) -> Result<MemoryReport, String>
{
    let mut report = MemoryReport {
        unit: if absolute_values { "b".into() } else { "kb".into() },
        ..Default::default()
    };


    // used as a factor for the unit
    let unit_division_factor: f64 = if absolute_values { 1.0 } else { 1024.0 };

    info!(target: "MemoryReport", "Retrieving memory usage of the application");

    // cache
    {
        let mut cache_report = CacheMemoryReport::default();

        info!(target: "MemoryReport", "Collecting cache informations...");
        let cache = cache_lock.read().await;

        let client_user = cache.get_client_user_mem_size() as f64 / unit_division_factor;
        cache_report.cache_total += client_user;
        cache_report.client_user = client_user;

        let application = cache.get_application_mem_size() as f64  / unit_division_factor;
        cache_report.cache_total += application;
        cache_report.application = application;

        let guilds = cache.get_guild_mem_size() as f64 / unit_division_factor;
        cache_report.cache_total += guilds;
        cache_report.application = guilds;

        let users = cache.get_users_mem_size() as f64 / unit_division_factor;
        cache_report.cache_total += users;
        cache_report.application = users;

        let channels = cache.get_channels_mem_size() as f64 / unit_division_factor;
        cache_report.cache_total += channels;
        cache_report.application = channels;

        report.total += cache_report.cache_total;


    }

    // shards
    {
        let mut shards_report = ShardMemoryReport::default();

        info!(target: "MemoryReport", "Collecting shard manager informations...");

        let shards = shard_manager.read().await;

        for (id, shard) in shards.get_shards().iter() {
            let mem = std::mem::size_of_val(shard) as f64 / unit_division_factor;
            shards_report.total += mem;
            shards_report.shards.insert(*id, mem);
        }

        drop(shards);

        report.total += shards_report.total;
        report.shards = shards_report;
    }

    info!(target: "MemoryReport", "The global usage of the Shard Manager & Cache Manager is '{t}' {u}.", t = report.total, u = report.unit);
    info!(target: "MemoryReport", "Report:\n{report:#?}");


    let formatted_report = match serde_json::to_string(&report) {
        Ok(s) => s,
        Err(e) => {
            error!(target: "MemoryReport", "Unable to convert informations to JSON: {e:#?}");
            return Err(format!("Unable to convert informations to JSON: {e:#?}"));
        }
    };


    // write the result
    let mut path = match std::env::current_dir() {
        Ok(mut p) => {
            p.push(config.memory_report_path);
            p
        },
        Err(e) => {
            error!(target: "MemoryReport", "Unable to find current dir: {e:#?}");
            return Err(format!("Unable to find current dir: {e:#?}"));
        }
    };

    // check if the folder "mem_report" exist
    {
        let metadata = match fs::metadata(path.clone()) {
            Ok(m) => m,
            Err(e) => {
                error!(target: "MemoryReport", "Unable to get metadata of {path:?}: {e:#?}");
                return Err(format!("Unable to get metadata of {path:?}: {e:#?}"));
            }
        };

        if !path.exists() || !metadata.is_dir() {
            // TODO make safe implementation
            if let Err(e) = fs::create_dir(&path) {
                error!(target: "MemoryReport", "Unable to create file {path:?}: {e:#?}");
                return Err(format!("Unable to create file {path:?}: {e:#?}"));
            }
        }
    }

    path.push(format!("{}.json", Utc::now().format("%Y-%m-%d_%H-%M-%S")));

    if let Err(e) = fs::write(path, formatted_report) {
        error!(target: "MemoryReport", "An error occured while writing the memory report informations: {e:#?}");
        return Err(format!("An error occured while writing the memory report informations: {e:#?}"));
    }

    info!(target: "MemoryReport", "Success of the operation");

    Ok(report)
}