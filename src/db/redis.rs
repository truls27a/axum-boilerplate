use redis::{Client, AsyncCommands};
use std::env;
use dotenv::dotenv;
use tracing::{info, warn, error, instrument};

#[derive(Clone)]
pub struct RedisStore {
    client: Client,
}

impl RedisStore {
    #[instrument]
    pub fn new() -> Result<Self, redis::RedisError> {
        dotenv().ok();
        let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
        info!(url = %redis_url, "Initializing Redis connection");
        
        match Client::open(redis_url) {
            Ok(client) => {
                info!("Redis client successfully created");
                Ok(Self { client })
            }
            Err(e) => {
                error!(error = %e, "Failed to create Redis client");
                Err(e)
            }
        }
    }

    #[instrument(skip(self, token))]
    pub async fn store_token(&self, user_id: i64, token: &str, expiry: i64) -> Result<(), redis::RedisError> {
        info!(user_id = %user_id, "Storing new authentication token");
        let mut con = match self.client.get_async_connection().await {
            Ok(con) => con,
            Err(e) => {
                error!(error = %e, "Failed to get Redis connection");
                return Err(e);
            }
        };
        
        // Store token with expiry
        let expiry_duration = (expiry - chrono::Utc::now().timestamp()) as u64;
        let key = format!("token:{}", token);
        let value = user_id.to_string();
        if let Err(e) = con.set_ex::<_, _, ()>(key, value, expiry_duration).await {
            error!(error = %e, "Failed to store token in Redis");
            return Err(e);
        }
        
        // Store user's active token
        let key = format!("user_token:{}", user_id);
        if let Err(e) = con.set::<_, _, ()>(key, token).await {
            error!(error = %e, "Failed to store user token reference in Redis");
            return Err(e);
        }
        
        info!(user_id = %user_id, "Successfully stored authentication token");
        Ok(())
    }

    #[instrument(skip(self, token))]
    pub async fn validate_token(&self, token: &str) -> Result<Option<i64>, redis::RedisError> {
        let mut con = match self.client.get_async_connection().await {
            Ok(con) => con,
            Err(e) => {
                error!(error = %e, "Failed to get Redis connection");
                return Err(e);
            }
        };
        
        // Get user_id associated with token
        let key = format!("token:{}", token);
        let user_id: Option<String> = match con.get::<_, Option<String>>(key).await {
            Ok(id) => {
                if id.is_none() {
                    warn!("Token not found in Redis");
                }
                id
            }
            Err(e) => {
                error!(error = %e, "Failed to validate token in Redis");
                return Err(e);
            }
        };
        
        let parsed_id = user_id.and_then(|id| id.parse::<i64>().ok());
        if let Some(id) = parsed_id {
            info!(user_id = %id, "Token successfully validated");
        }
        
        Ok(parsed_id)
    }

    #[instrument(skip(self, token))]
    pub async fn invalidate_token(&self, token: &str) -> Result<(), redis::RedisError> {
        info!("Invalidating authentication token");
        let mut con = match self.client.get_async_connection().await {
            Ok(con) => con,
            Err(e) => {
                error!(error = %e, "Failed to get Redis connection");
                return Err(e);
            }
        };
        
        // Get user_id associated with token
        if let Some(user_id) = self.validate_token(token).await? {
            // Remove user's active token
            let key = format!("user_token:{}", user_id);
            if let Err(e) = con.del::<_, ()>(key).await {
                error!(error = %e, user_id = %user_id, "Failed to remove user token reference");
                return Err(e);
            }
            info!(user_id = %user_id, "Removed user token reference");
        }
        
        // Remove token
        let key = format!("token:{}", token);
        if let Err(e) = con.del::<_, ()>(key).await {
            error!(error = %e, "Failed to remove token");
            return Err(e);
        }
        
        info!("Successfully invalidated token");
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn invalidate_user_tokens(&self, user_id: i64) -> Result<(), redis::RedisError> {
        info!(user_id = %user_id, "Invalidating all user tokens");
        let mut con = match self.client.get_async_connection().await {
            Ok(con) => con,
            Err(e) => {
                error!(error = %e, "Failed to get Redis connection");
                return Err(e);
            }
        };
        
        // Get user's active token
        let key = format!("user_token:{}", user_id);
        let token: Option<String> = match con.get::<_, Option<String>>(key.clone()).await {
            Ok(token) => token,
            Err(e) => {
                error!(error = %e, "Failed to get user token");
                return Err(e);
            }
        };
        
        if let Some(token) = token {
            // Remove token
            let token_key = format!("token:{}", token);
            if let Err(e) = con.del::<_, ()>(token_key).await {
                error!(error = %e, "Failed to remove token");
                return Err(e);
            }
            info!("Removed user token");
        }
        
        // Remove user's active token reference
        if let Err(e) = con.del::<_, ()>(key).await {
            error!(error = %e, "Failed to remove user token reference");
            return Err(e);
        }
        
        info!(user_id = %user_id, "Successfully invalidated all user tokens");
        Ok(())
    }
} 