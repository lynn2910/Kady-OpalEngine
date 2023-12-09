use std::sync::Arc;
use std::time::Duration;
use log::error;
use sqlx::MySqlPool;
use tokio::time::Instant;
use database::Database;
use database::dynamic_requests::DynamicRequest;

pub(crate) fn database_cleaner(database: Database) {
    tokio::spawn(async move {
        let database = Arc::new(database);
        loop {
            let start_time = Instant::now();
            let pool = database.get_pool().await;
            let requests = database.get_requests().await;

            clear_cookies_quiz(&pool, &requests).await;

            tokio::time::sleep(Duration::from_secs(5 * 60) - (Instant::now() - start_time)).await;
        }
    });
}

async fn clear_cookies_quiz(pool: &MySqlPool, requests: &DynamicRequest) {
    if let Err(e) = sqlx::query(requests.system.quiz.clear_users.as_str()).execute(pool).await {
        error!(target: "Runtime", "An error occured while cleaning the users from the cookies quiz table: {e:#?}");
    }
}