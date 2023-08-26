use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use axum::routing::get;
use log::{error, info};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::{Mutex, RwLock};
use client::manager::cache::CacheManager;
use client::manager::shard::ShardManager;
use client::typemap::Type;
use database::Database;

async fn home() -> String {
    "Hello World".into()
}

pub fn start(
    docker: &mut Api,
    host: &str
)
{
    let (tx, mut rx): (UnboundedSender<()>, UnboundedReceiver<()>) = tokio::sync::mpsc::unbounded_channel();
    let host = SocketAddr::from_str( host).expect("Bro, learn how to write host & port informations for the API");

    let _ = docker.stop_channel.insert(Arc::new(tx));

    let state_clone = docker.state.clone();
    tokio::spawn(async move {
        let app = axum::routing::Router::new()
            .route("/", get(home))
            .with_state(state_clone);

        let server = hyper::Server::bind(&host)
            .serve(app.into_make_service())
            .with_graceful_shutdown(async { rx.recv().await.unwrap_or(()) });

        info!(target: "ApiCore", "API is listening on {host:?}");

        let exit_status = server.await;

        if let Err(e) = exit_status {
            error!(target: "ApiCore", "The API exited with an error: {e:#?}")
        }
        info!(target: "ApiCore", "The API exited properly")
    });
}

#[derive(Clone)]
pub struct Api {
    state: AppState,
    /// Send `()` to this channel for shutting down the API
    stop_channel: Option<Arc<UnboundedSender<()>>>
}

impl Api {
    pub fn new(state: AppState) -> Self {
        Self {
            stop_channel: None,
            state
        }
    }

    /// Stop the API
    pub async fn stop(&mut self) -> Option<()> {
        if let Some(chl) = self.stop_channel.take() {
            chl.send(()).ok()
        } else {
            None
        }
    }
}

impl Type for Api {
    type Value = Self;
}

pub type AppState = Arc<ApiState>;

#[derive(Clone)]
/// Contain every informations useful for the API
pub struct ApiState {
    /// The current status of the API
    pub(crate) status: ApiStatus,
    cache: Arc<RwLock<CacheManager>>,
    sec_container: SecurityContainer
}

impl ApiState {
    pub fn new(cache: Arc<RwLock<CacheManager>>, sec: SecurityContainer) -> Self {
        Self { status: ApiStatus::default(), cache, sec_container: sec }
    }
}

#[derive(Clone, Debug)]
pub struct SecurityContainer {
    /// The shard manager protected by a Arc & Read-Write Lock
    shard_manager: Arc<RwLock<ShardManager>>,
    /// The database
    database: Database
}

impl SecurityContainer {
    pub fn new(shard_manager: Arc<RwLock<ShardManager>>, database: Database) -> Self {
        Self { shard_manager, database }
    }
}

/// Define what is the status of the API
#[derive(Default, Debug, Clone, Copy)]
pub enum ApiStatus {
    #[default]
    Starting,
    Online,
    Maintenance,
    Offline
}