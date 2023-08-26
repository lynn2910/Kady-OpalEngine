use chrono::{DateTime, Utc};
use sqlx::MySqlPool;
use error::{DatabaseError, Error, Result};
use crate::constants::USER_LIFETIME;

/// Represent a user in the database
#[derive(sqlx::FromRow, Debug)]
pub struct User {
    /// The ID of the User
    pub id: String,
    /// The last time the player had been on a server
    pub last_seen: DateTime<Utc>,
    /// The last time a data was modified for this user
    pub last_edited_timestamp: DateTime<Utc>,
    /// If the user has authorized private messages by the bot
    pub send_private_messages: bool,

    /// The xp of the player
    /// Accessible only when the table `users_xp` is joined
    pub xp: Option<i64>,
    /// The level of the player
    /// Accessible only when the table `users_xp` is joined
    pub lvl: Option<i64>,

    /// The badges of the user
    /// Accessible only when the table `user_badges` is joined
    pub badge: Option<u64>,

    /// The biography of the user
    /// Accessible only when the table `user_biography` is joined
    pub biography: Option<String>
}

//const GET_USER_QUERY: &str = r#"SELECT * FROM users
//    LEFT JOIN user_badges ON users.id = user_badges.user
//    LEFT JOIN user_biography ON users.id = user_biography.user
//    LEFT JOIN user_xp ON users.id = user_xp.user
//    WHERE users.id = '782164174821523467';"#;

impl User {
    /// Get a user from the database
    pub async fn from_pool<T: ToString>(pool: &MySqlPool, request: &str, id: T) -> Result<User> {
        let query = sqlx::query_as::<_, Self>(request)
            .bind(id.to_string());

        match query.fetch_one(pool).await {
            Ok(user) => Ok(user),
            Err(e) => Err(Error::Database(DatabaseError::QueryError(e.to_string())))
        }
    }

    /// Create a user in the database
    pub async fn create(pool: &MySqlPool, request: &str, id: String) -> Result<()> {
        let query = sqlx::query(request)
            .bind(id);

        match query.execute(pool).await {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::Database(DatabaseError::QueryError(e.to_string())))
        }
    }

    /// Ensure that a user exists in the database
    pub async fn ensure(pool: &MySqlPool, request: &str, id: String) -> Result<()> {
        let query = sqlx::query(request)
            .bind(id.clone());

        match query.execute(pool).await {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::Database(DatabaseError::QueryError(e.to_string())))
        }
    }

    /// Update the last seen of a user
    pub async fn update_last_seen(pool: &MySqlPool, request: &str, id: String) -> Result<()> {
        let query = sqlx::query(request)
            .bind(Utc::now())
            .bind(id);

        match query.execute(pool).await {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::Database(DatabaseError::QueryError(e.to_string())))
        }
    }

    /// Update the last edited timestamp of a user
    pub async fn update_last_edited_timestamp(pool: &MySqlPool, request: &str, id: String) -> Result<()> {
        let query = sqlx::query(request)
            .bind(Utc::now())
            .bind(id);

        match query.execute(pool).await {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::Database(DatabaseError::QueryError(e.to_string())))
        }
    }

    /// Check if the user can be deleted
    pub async fn can_be_deleted(pool: &MySqlPool, request: &str, id: String) -> Result<bool> {
        let query = Self::from_pool(pool, request, id.clone()).await?;

        // check if last_seen was 14 days ago
        let now = Utc::now().timestamp();
        let last_seen = query.last_seen.timestamp();

        Ok(now - last_seen >= USER_LIFETIME)
    }
}

/// Represent a marriage
#[derive(sqlx::FromRow, Debug)]
pub struct Marriage {
    pub user1: String,
    pub user2: String,
    /// The timestamp when the two have been married
    pub timestamp: DateTime<Utc>
}

impl Marriage {
    pub async fn from_pool(pool: &MySqlPool, request: &str, id: String) -> Result<Option<Marriage>> {
        let query = sqlx::query_as::<_, Self>(request)
            .bind(id.clone())
            .bind(id);

        match query.fetch_optional(pool).await {
            Ok(marriage) => Ok(marriage),
            Err(e) => Err(Error::Database(DatabaseError::QueryError(e.to_string())))
        }
    }
}

/// Represent a reputation that was given
#[derive(sqlx::FromRow, Debug)]
pub struct Reputation {
    /// The ID of the player who gave the reputation
    pub user_from: String,
    /// The ID of the reputation who has received the point
    pub user_to: String,
    /// The timestamp when the point was given
    pub timestamp: DateTime<Utc>,
    /// The guild where the point was given
    pub guild: String
}

#[derive(sqlx::FromRow, Debug)]
pub struct ReputationRanking {
    pub user_to: String,
    pub cookies: i64,
}

#[derive(sqlx::FromRow, Debug)]
pub struct ReputationRankingRank {
    pub user_to: String,
    pub cookies: i64,
    pub user_rank: i64,
}

impl Reputation {
    /// Get all the reputation that a user has received
    pub async fn get_reputation<T: ToString>(pool: &MySqlPool, request: &str, user: T) -> Result<Vec<Self>> {
        let query = sqlx::query_as::<_, Self>(request)
            .bind(user.to_string());

        match query.fetch_all(pool).await {
            Ok(reputations) => Ok(reputations),
            Err(e) => Err(Error::Database(DatabaseError::QueryError(e.to_string())))
        }
    }

    /// Get the number of reputation that a user has received
    pub async fn get_reputation_count<T: ToString>(pool: &MySqlPool, request: &str, user: T) -> Result<usize> {
        match Self::get_reputation(pool, request, user).await {
            Ok(reputations) => Ok(reputations.len()),
            Err(e) => Err(e)
        }
    }

    /// Get the last reputation that a user has received if any
    pub async fn get_last_reputation<T: ToString>(pool: &MySqlPool, request: &str, user: T) -> Result<Option<Self>> {
        let query = sqlx::query_as::<_, Self>(request)
            .bind(user.to_string());

        match query.fetch_optional(pool).await {
            Ok(reputation) => Ok(reputation),
            Err(e) => Err(Error::Database(DatabaseError::QueryError(e.to_string())))
        }
    }

    pub async fn get_in_guild_reputation<T: ToString>(pool: &MySqlPool, request: &str, user: T, guild: T) -> Result<Vec<Self>> {
        let query = sqlx::query_as::<_, Self>(request)
            .bind(user.to_string())
            .bind(guild.to_string());

        match query.fetch_all(pool).await {
            Ok(reputation) => Ok(reputation),
            Err(e) => Err(Error::Database(DatabaseError::QueryError(e.to_string())))
        }
    }
}