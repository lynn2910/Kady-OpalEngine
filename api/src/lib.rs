mod root;
mod public_dispatcher;

use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use axum::response::IntoResponse;
use axum::routing::get;
use hyper::StatusCode;
use log::{error, info};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::RwLock;
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
            .route("/public/:file_name", get(public_dispatcher::handler))
            .route("/root/private/socket", get(root::handler))
            .with_state(state_clone)
            .fallback(fallback_handler);

        let server = hyper::Server::bind(&host)
            .serve(app.into_make_service_with_connect_info::<SocketAddr>())
            .with_graceful_shutdown(async { rx.recv().await.unwrap_or(()) });

        info!(target: "ApiCore", "API is listening on {host:?}");

        let exit_status = server.await;

        if let Err(e) = exit_status {
            error!(target: "ApiCore", "The API exited with an error: {e:#?}")
        }
        info!(target: "ApiCore", "The API exited properly")
    });
}

async fn fallback_handler() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Cannot find this route, are you in the right place?")
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
#[allow(dead_code)]
/// Contain every informations useful for the API
pub struct ApiState {
    /// The current status of the API
    pub(crate) status: ApiStatus,  //        name,   path,   type
    pub(crate) public_files: Arc<RwLock<Vec<(String, String, String)>>>,
    cache: Arc<RwLock<CacheManager>>,
    sec_container: SecurityContainer
}

impl ApiState {
    pub fn new(
        cache: Arc<RwLock<CacheManager>>,
        sec_container: SecurityContainer,
        public_files: Arc<RwLock<Vec<(String, String, String)>>>
    ) -> Self {
        Self { status: ApiStatus::default(), cache, sec_container, public_files }
    }
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
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