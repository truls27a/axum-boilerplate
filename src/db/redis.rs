use dotenv::dotenv;
use redis::{Client, RedisError};
use redis::aio::{ConnectionManager, ConnectionLike};
use redis::AsyncCommands;
use std::env;
use std::time::Duration;
use tracing::{error, info, instrument};

#[derive(Clone)]
pub struct RedisStore {
    client: Client,
}

impl RedisStore {
    const ALLOWLIST_PREFIX: &str = "refresh_allowlist:";
    const BLACKLIST_PREFIX: &str = "token_blacklist:";

    #[instrument]
    pub fn new() -> Result<Self, RedisError> {
        dotenv().ok();
        let redis_url = env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
        info!(url = %redis_url, "Initializing Redis connection");

        Client::open(redis_url).map(|client| Self { client })
    }

    async fn conn(&self) -> Result<ConnectionManager, RedisError> {
        ConnectionManager::new(self.client.clone()).await
    }

    /* ----------  ALLOWLIST  (for refresh tokens) ---------- */

    pub async fn add_to_allowlist(
        &self,
        jti: &str,
        user_id: i64,
        ttl_secs: usize,
    ) -> Result<(), RedisError> {
        let key = format!("{}{}", Self::ALLOWLIST_PREFIX, jti);
        let mut con = self.conn().await?;
        con.set_ex::<_, _, ()>(key, user_id, ttl_secs).await
    }

    pub async fn remove_from_allowlist(&self, jti: &str) -> Result<(), RedisError> {
        let key = format!("{}{}", Self::ALLOWLIST_PREFIX, jti);
        let mut con = self.conn().await?;
        con.del::<_, ()>(key).await
    }

    pub async fn is_allowlisted(&self, jti: &str) -> redis::RedisResult<bool> {
        let key = format!("{}{}", Self::ALLOWLIST_PREFIX, jti);
        let mut con = self.conn().await?;
        con.exists(key).await
    }

    /* ----------  BLACKLIST  (for access OR refresh) ---------- */

    pub async fn blacklist_token(&self, token: &str, ttl_secs: usize) -> Result<(), RedisError> {
        let key = format!("{}{}", Self::BLACKLIST_PREFIX, token);
        let mut con = self.conn().await?;
        con.set_ex::<_, _, ()>(key, 1u8, ttl_secs).await
    }

    pub async fn is_blacklisted(&self, token: &str) -> redis::RedisResult<bool> {
        let key = format!("{}{}", Self::BLACKLIST_PREFIX, token);
        let mut con = self.conn().await?;
        con.exists(key).await
    }
}
