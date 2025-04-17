use redis::{Client, AsyncCommands};
use std::env;
use dotenv::dotenv;

#[derive(Clone)]
pub struct RedisStore {
    client: Client,
}

impl RedisStore {
    pub fn new() -> Result<Self, redis::RedisError> {
        dotenv().ok();
        let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
        let client = Client::open(redis_url)?;
        Ok(Self { client })
    }

    pub async fn store_token(&self, user_id: i64, token: &str, expiry: i64) -> Result<(), redis::RedisError> {
        let mut con = self.client.get_async_connection().await?;
        
        // Store token with expiry
        let expiry_duration = (expiry - chrono::Utc::now().timestamp()) as u64;
        let _: () = con.set_ex(format!("token:{}", token), user_id.to_string(), expiry_duration).await?;
        
        // Store user's active token
        let _: () = con.set(format!("user_token:{}", user_id), token).await?;
        
        Ok(())
    }

    pub async fn validate_token(&self, token: &str) -> Result<Option<i64>, redis::RedisError> {
        let mut con = self.client.get_async_connection().await?;
        
        // Get user_id associated with token
        let user_id: Option<String> = con.get(format!("token:{}", token)).await?;
        
        Ok(user_id.and_then(|id| id.parse::<i64>().ok()))
    }

    pub async fn invalidate_token(&self, token: &str) -> Result<(), redis::RedisError> {
        let mut con = self.client.get_async_connection().await?;
        
        // Get user_id associated with token
        if let Some(user_id) = self.validate_token(token).await? {
            // Remove user's active token
            let _: () = con.del(format!("user_token:{}", user_id)).await?;
        }
        
        // Remove token
        let _: () = con.del(format!("token:{}", token)).await?;
        
        Ok(())
    }

    pub async fn invalidate_user_tokens(&self, user_id: i64) -> Result<(), redis::RedisError> {
        let mut con = self.client.get_async_connection().await?;
        
        // Get user's active token
        let token: Option<String> = con.get(format!("user_token:{}", user_id)).await?;
        
        if let Some(token) = token {
            // Remove token
            let _: () = con.del(format!("token:{}", token)).await?;
        }
        
        // Remove user's active token reference
        let _: () = con.del(format!("user_token:{}", user_id)).await?;
        
        Ok(())
    }
} 