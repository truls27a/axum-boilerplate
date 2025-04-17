use jsonwebtoken::{encode, decode, EncodingKey, DecodingKey, Header, Validation, errors::Error as JwtError};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};
use dotenv::dotenv;
use std::env;
use crate::db::RedisStore;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64, // user id
    pub exp: i64, // expiration time
    pub iat: i64, // issued at
}

impl Claims {
    pub fn new(user_id: i64) -> Self {
        let now = Utc::now();
        let expires_at = now + Duration::hours(24);
        
        Self {
            sub: user_id,
            exp: expires_at.timestamp(),
            iat: now.timestamp(),
        }
    }
}

pub async fn create_token(user_id: i64, redis_store: &RedisStore) -> Result<String, JwtError> {
    let claims = Claims::new(user_id);

    // Load .env file
    dotenv().ok();

    let secret = env::var("SECRET_KEY").expect("SECRET_KEY must be set");

    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))?;
    
    // Store token in Redis
    redis_store.store_token(user_id, &token, claims.exp)
        .await
        .map_err(|_| JwtError::from(jsonwebtoken::errors::ErrorKind::InvalidToken))?;

    Ok(token)
}

pub async fn decode_token(token: &str, redis_store: &RedisStore) -> Result<Claims, JwtError> {
    dotenv().ok();
    let secret = env::var("SECRET_KEY").expect("SECRET_KEY must be set");
    
    // First validate token in Redis
    let user_id = redis_store.validate_token(token)
        .await
        .map_err(|_| JwtError::from(jsonwebtoken::errors::ErrorKind::InvalidToken))?
        .ok_or_else(|| JwtError::from(jsonwebtoken::errors::ErrorKind::InvalidToken))?;
    
    // Then decode JWT
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default()
    )?;
    
    // Verify the user_id matches
    if token_data.claims.sub != user_id {
        return Err(JwtError::from(jsonwebtoken::errors::ErrorKind::InvalidToken));
    }
    
    Ok(token_data.claims)
}

pub async fn invalidate_token(token: &str, redis_store: &RedisStore) -> Result<(), JwtError> {
    redis_store.invalidate_token(token)
        .await
        .map_err(|_| JwtError::from(jsonwebtoken::errors::ErrorKind::InvalidToken))
} 