use log::error;
use client::manager::http::Http;
use client::models::message::MessageBuilder;
use client::models::user::UserId;
use translation::fmt::formatter::Formatter;
use translation::message;

pub(crate) mod nuggets {
    use std::ops::Deref;
    use std::sync::Arc;
    use std::time::Duration;
    use log::error;
    use tokio::sync::RwLock;
    use tokio::time::sleep;
    use client::manager::cache::CacheManager;
    use client::manager::http::Http;
    use client::models::user::UserId;
    use database::Database;
    use database::model::users::{CookiesNumber, UserNuggets};
    use crate::crates::cookies::notify_cookies_given_from_system;


    /// This thread will run every hours a check to see if a users have enough nuggets to get one or multiple cookies
    pub fn nugget_updater_task(
        database: Database,
        http: Arc<Http>,
        cache: Arc<RwLock<CacheManager>>
    )
    {
        tokio::spawn(async move {
            'main: loop {
                let pool = database.get_pool().await;
                let pool = pool.deref();
                let requests = database.get_requests().await;


                let client_user = {
                    let cache = cache.read().await;

                    match cache.get_client_user() {
                        Some(c) => c.clone(),
                        _ => match http.fetch_client_user().await {
                            Ok(Ok(c)) => c,
                            Ok(Err(e)) => {
                                error!(target: "NuggetUpdater", "An error occured while fetching the client user: {e:#?}");
                                continue 'main;
                            }
                            Err(e) => {
                                error!(target: "NuggetUpdater", "Cannot fetch the client user: {e:#?}");
                                continue 'main;
                            }
                        }
                    }
                };


                let query = sqlx::query_as::<_, UserNuggets>(requests.users.cookies.get_updatable_nuggets.as_str())
                    .fetch_all(pool)
                    .await;

                let updatable_users = match query {
                    Ok(rows) => rows,
                    Err(e) => {
                        error!(target: "NuggetUpdater", "An error occured while fetching updatable user nuggets: {e:#?}");
                        continue 'main;
                    }
                };

                if !updatable_users.is_empty() {
                    for row in updatable_users.iter() {
                        let new_cookies = (row.nuggets - row.nuggets % 6) / 6;
                        let nuggets_after = row.nuggets - (new_cookies * 6);

                        if new_cookies > 0 && nuggets_after != row.nuggets {
                            // update the user's cookies && nuggets

                            // give cookies
                            for _ in 0..new_cookies {
                                let q = sqlx::query(requests.users.cookies.give_cookie.as_str())
                                    .bind(client_user.id.to_string())
                                    .bind(&row.user)
                                    .execute(pool)
                                    .await;

                                if let Err(e) = q {
                                    error!(target: "NuggetUpdater", "Cannot add a cookie to '{}': {e:#?}", row.user)
                                }
                            }

                            // set the new nugget number
                            let q = sqlx::query(requests.users.cookies.decrease_nuggets.as_str())
                                .bind(nuggets_after)
                                .bind(&row.user)
                                .execute(pool)
                                .await;

                            if let Err(e) = q {
                                error!(target: "NuggetUpdater", "Cannot set the number of nuggets for '{}' to '{nuggets_after}' nuggets: {e:#?}", row.user)
                            }

                            {
                                let q = sqlx::query_as::<_, CookiesNumber>(requests.users.cookies.get_cookies_number.as_str())
                                    .bind(&row.user)
                                    .fetch_one(pool)
                                    .await;

                                match q {
                                    Ok(cookies) => {
                                        notify_cookies_given_from_system(
                                            &http,
                                            UserId::from(row.user.clone()),
                                            new_cookies,
                                            cookies.count as u64
                                        ).await;
                                    }
                                    Err(e) => {
                                        error!(target: "NuggetUpdater", "An error occured while fetching all cookies from '{}': {e:#?}", row.user)
                                    }
                                }
                            }
                        }
                    }
                }

                // 86400000ms = 1d
                sleep(Duration::from_millis(86400000)).await;
            }
        });
    }
}

pub mod quiz {
    use sqlx::{Error, MySqlPool};
    use sqlx::mysql::MySqlQueryResult;
    use strsim::levenshtein;
    use database::dynamic_requests::DynamicRequest;

    #[derive(sqlx::FromRow, Clone, Debug)]
    pub struct Quiz {
        pub id: String,
        pub category: String
    }

    #[derive(sqlx::FromRow, Clone, Debug)]
    pub struct UserQuizQuestion {
        pub id: String,
        pub user: String,
        pub completed: bool
    }

    #[derive(sqlx::FromRow, Clone, Debug)]
    pub struct UserQuizAnswer {
        pub id: String,
        pub answer: String
    }

    pub async fn get_random_question(pool: &MySqlPool, requests: &DynamicRequest) -> Result<Quiz, Error> {
        sqlx::query_as::<_, Quiz>(requests.system.quiz.get_question_random.as_str())
            .fetch_one(pool)
            .await
    }

    #[allow(dead_code)]
    pub async fn get_question(pool: &MySqlPool, requests: &DynamicRequest, id: impl ToString) -> Result<Quiz, Error> {
        sqlx::query_as::<_, Quiz>(requests.system.quiz.get_question.as_str())
            .bind(id.to_string())
            .fetch_one(pool)
            .await
    }

    #[allow(dead_code)]
    pub async fn get_random_question_without_last(pool: &MySqlPool, requests: &DynamicRequest, last: impl ToString) -> Result<Quiz, Error> {
        sqlx::query_as::<_, Quiz>(requests.system.quiz.get_question_random_without_last.as_str())
            .bind(last.to_string())
            .fetch_one(pool)
            .await
    }

    pub async fn get_user(pool: &MySqlPool, requests: &DynamicRequest, user_id: impl ToString) -> Result<Option<UserQuizQuestion>, Error> {
        sqlx::query_as::<_, UserQuizQuestion>(requests.system.quiz.get_user.as_str())
            .bind(user_id.to_string())
            .fetch_optional(pool)
            .await
    }

    pub async fn insert_user(pool: &MySqlPool, requests: &DynamicRequest, user: impl ToString, question_id: impl ToString) -> Result<MySqlQueryResult, Error> {
        sqlx::query(requests.system.quiz.insert_user.as_str())
            .bind(question_id.to_string())
            .bind(user.to_string())
            .execute(pool)
            .await
    }

    #[allow(dead_code)]
    pub async fn update_user_question(pool: &MySqlPool, requests: &DynamicRequest, user: impl ToString, question: impl ToString) -> Result<MySqlQueryResult, Error> {
        sqlx::query(requests.system.quiz.update_user_question.as_str())
            .bind(question.to_string())
            .bind(user.to_string())
            .execute(pool)
            .await
    }

    pub async fn question_completed(pool: &MySqlPool, requests: &DynamicRequest, user: impl ToString) -> Result<MySqlQueryResult, Error> {
        sqlx::query(requests.system.quiz.question_completed.as_str())
            .bind(user.to_string())
            .execute(pool)
            .await
    }

    pub async fn get_all_possible_answers(pool: &MySqlPool, requests: &DynamicRequest, id: impl ToString) -> Result<Vec<UserQuizAnswer>, Error> {
        sqlx::query_as::<_, UserQuizAnswer>(requests.system.quiz.get_all_possible_answers.as_str())
            .bind(id.to_string())
            .fetch_all(pool)
            .await
    }

    pub fn check_answer_validity(answer: &str, chunk: &Vec<UserQuizAnswer>, tolerance: usize) -> bool {
        for awsr in chunk {
            let distance = levenshtein(answer, awsr.answer.to_lowercase().as_str());
            if distance <= tolerance { return true }
        }
        false
    }
}

pub async fn notify_new_cookie(
    http: &Http,
    from_name: String,
    to: UserId,
    cookies_given: u64,
    cookies_count: u64
)
{
    let dm_channel = match http.create_dm_channel(&to).await {
        Ok(Ok(c)) => c,
        Ok(Err(e)) => {
            error!(target: "CookieNotifier", "An error occured while fetching the DM channel: {e:#?}");
            return;
        }
        Err(e) => {
            error!(target: "CookieNotifier", "An error occured while fetching the DM channel: {e:#?}");
            return;
        }
    };

    let msg = dm_channel.id.send_message(
        http,
        MessageBuilder::new()
            .set_content(
                message!(
                    "fr",
                    "features::cookies::new_cookie_notification",
                    Formatter::new()
                        .add("user", from_name)
                        .add("new", cookies_given)
                        .add("cookies", cookies_count)
                )
            )
    ).await;

    if let Err(e) = msg {
        error!(target: "CookieNotifier", "Cannot notify new cookies to {to}: {e:#?}");
    }
}

pub async fn notify_cookies_given_from_system(
    http: &Http,
    to: UserId,
    cookies_given: u64,
    cookies_count: u64
)
{
    let dm_channel = match http.create_dm_channel(&to).await {
        Ok(Ok(c)) => c,
        Ok(Err(e)) => {
            error!(target: "CookieNotifier", "An error occured while fetching the DM channel: {e:#?}");
            return;
        }
        Err(e) => {
            error!(target: "CookieNotifier", "An error occured while fetching the DM channel: {e:#?}");
            return;
        }
    };

    let msg = dm_channel.id.send_message(
        http,
        MessageBuilder::new()
            .set_content(
                message!(
                    "fr",
                    "features::cookies::new_cookies_from_nuggets",
                    Formatter::new()
                        .add("new", cookies_given)
                        .add("cookies", cookies_count)
                )
            )
    ).await;

    if let Err(e) = msg {
        error!(target: "CookieNotifier", "Cannot notify new cookies to {to}: {e:#?}");
    }
}

#[allow(dead_code)]
pub async fn notify_cookies_given_from_admin(
    http: &Http,
    admin_name: String,
    to: UserId,
    cookies_given: u64,
    cookies_count: u64
)
{
    let dm_channel = match http.create_dm_channel(&to).await {
        Ok(Ok(c)) => c,
        Ok(Err(e)) => {
            error!(target: "CookieNotifier", "An error occured while fetching the DM channel: {e:#?}");
            return;
        }
        Err(e) => {
            error!(target: "CookieNotifier", "An error occured while fetching the DM channel: {e:#?}");
            return;
        }
    };

    let msg = dm_channel.id.send_message(
        http,
        MessageBuilder::new()
            .set_content(
                message!(
                    "fr",
                    "features::cookies::new_cookies_from_nuggets",
                    Formatter::new()
                        .add("admin", admin_name)
                        .add("new", cookies_given)
                        .add("cookies", cookies_count)
                )
            )
    ).await;

    if let Err(e) = msg {
        error!(target: "CookieNotifier", "Cannot notify new cookies to {to}: {e:#?}");
    }
}