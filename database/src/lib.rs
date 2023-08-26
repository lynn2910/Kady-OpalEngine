pub mod model;
mod constants;
pub mod dynamic_requests;

use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use serde_json::Value;
use sqlx::{MySql, MySqlPool};
use sqlx::pool::PoolConnection;
use archive::Archive;
use client::typemap::Type;
use error::{DatabaseError, Error, Result};
use tokio::sync::{RwLock, RwLockReadGuard};
use config::Config;

#[derive(Clone, Debug)]
pub struct Database {
    pool: Arc<RwLock<MySqlPool>>,
    requests: Arc<RwLock<dynamic_requests::DynamicRequest>>
}

impl Type for Database {
    type Value = Arc<Database>;
}

impl Database {
    pub async fn connect(archive: &Archive, config: &Config) -> Result<Self> {
        Ok(Self {
            pool: Arc::new(RwLock::new(connect(archive).await?)),
            requests: Arc::new(
                RwLock::new(
                    dynamic_requests::DynamicRequest::from_file(
                        PathBuf::from_str(config.dynamic_requests.as_str()).expect("Cannot convert dynamic_requests path to PathBuf")
                    )?
                )
            )
        })
    }

    pub async fn update_requests(&mut self, config: &Config) -> core::result::Result<(), String> {
        let mut requests = self.requests.write().await;

        match PathBuf::from_str(config.dynamic_requests.as_str()) {
            Ok(p) => {
                match dynamic_requests::DynamicRequest::from_file(p) {
                    Ok(dr) => {
                        *requests = dr;
                        Ok(())
                    }
                    Err(e) => Err(e.to_string())
                }
            },
            Err(_) => Err("Cannot convert dynamic_requests path to PathBuf".to_string())
        }
    }

    /// Get the pool as a reference
    pub async fn get_pool(&self) -> RwLockReadGuard<MySqlPool> {
        self.pool.as_ref().read().await
    }

    /// Acquire a connection from the pool
    pub async fn get_connection(&self) -> Result<PoolConnection<MySql>> {
        let conn = self.pool.read().await
            .acquire()
            .await;

        match conn {
            Ok(c) => Ok(c),
            Err(e) => Err(Error::Database(DatabaseError::CannotAcquireConnection(e.to_string())))
        }
    }

    /// Get the dynamic requests as a reference
    pub async fn get_requests(&self) -> RwLockReadGuard<dynamic_requests::DynamicRequest> {
        self.requests.as_ref().read().await
    }
}

/// Create a pool with the archive given
async fn connect(archive: &Archive) -> Result<MySqlPool> {
    match MySqlPool::connect(prepare_connection(archive)?.as_str()).await {
        Ok(pool) => Ok(pool),
        Err(e) => Err(Error::Database(DatabaseError::CannotConnect(e.to_string())))
    }
}

/// Get an archive and format the connection url for the database
fn prepare_connection(archive: &Archive) -> Result<String> {
    // get credentials
    let db_credentials = if let Some(v) = archive.get::<Value>("database") { v }
        else { return Err(Error::Database(DatabaseError::InvalidCredentials("Field 'database' is missing in the archive".into()))) };

    // get each field required
    let username = if let Some(uname) = db_credentials.get("username") { value_to_string(uname)? }
        else { return Err(Error::Database(DatabaseError::InvalidCredentials("Field 'username' is missing in the archive".into()))) };

    let password = if let Some(pwd) = db_credentials.get("password") { value_to_string(pwd)? }
        else { return Err(Error::Database(DatabaseError::InvalidCredentials("Field 'password' is missing in the archive".into()))) };

    let host = if let Some(h) = db_credentials.get("host") { value_to_string(h)? }
        else { return Err(Error::Database(DatabaseError::InvalidCredentials("Field 'host' is missing in the archive".into()))) };

    let db_name = if let Some(n) = db_credentials.get("db_name") { value_to_string(n)? }
        else { return Err(Error::Database(DatabaseError::InvalidCredentials("Field 'db_name' is missing in the archive".into()))) };

    Ok(format!("mysql://{username}:{password}@{host}/{db_name}"))
}

/// Transform a JSON value into string
fn value_to_string(v: &Value) -> Result<String> {
    if let Some(e) = v.as_str() {
        Ok(e.to_string())
    } else {
        Err(Error::Database(DatabaseError::InvalidCredentials(format!("Value {v:?} cannot be mode to string"))))
    }
}
