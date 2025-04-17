// TODO: Add refresh tokens

use jsonwebtoken::{encode, decode, EncodingKey, DecodingKey, Header, Validation, errors::Error as JwtError};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};
use dotenv::dotenv;
use std::env;
use crate::db::RedisStore;

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessClaims {
    pub sub: i64, // user id
    pub exp: i64, // expiration time
    pub iat: i64, // issued at
    pub token_type: String, // token type (access)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshClaims {
    pub sub: i64, // user id
    pub exp: i64, // expiration time
    pub iat: i64, // issued at
    pub token_type: String, // token type (refresh)
}

impl AccessClaims {
    pub fn new(user_id: i64) -> Self {
        let now = Utc::now();
        let expires_at = now + Duration::minutes(15); // Access tokens expire in 15 minutes
        
        Self {
            sub: user_id,
            exp: expires_at.timestamp(),
            iat: now.timestamp(),
            token_type: "access".to_string(),
        }
    }
}

impl RefreshClaims {
    pub fn new(user_id: i64) -> Self {
        let now = Utc::now();
        let expires_at = now + Duration::days(7); // Refresh tokens expire in 7 days
        
        Self {
            sub: user_id,
            exp: expires_at.timestamp(),
            iat: now.timestamp(),
            token_type: "refresh".to_string(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
}

pub async fn create_token_pair(user_id: i64, redis_store: &RedisStore) -> Result<TokenPair, JwtError> {
    let access_claims = AccessClaims::new(user_id);
    let refresh_claims = RefreshClaims::new(user_id);

    dotenv().ok();
    let secret = env::var("SECRET_KEY").expect("SECRET_KEY must be set");

    let access_token = encode(
        &Header::default(),
        &access_claims,
        &EncodingKey::from_secret(secret.as_bytes())
    )?;

    let refresh_token = encode(
        &Header::default(),
        &refresh_claims,
        &EncodingKey::from_secret(secret.as_bytes())
    )?;

    // Store both tokens in Redis
    redis_store.store_token(user_id, &access_token, access_claims.exp)
        .await
        .map_err(|_| JwtError::from(jsonwebtoken::errors::ErrorKind::InvalidToken))?;

    redis_store.store_refresh_token(user_id, &refresh_token, refresh_claims.exp)
        .await
        .map_err(|_| JwtError::from(jsonwebtoken::errors::ErrorKind::InvalidToken))?;

    Ok(TokenPair {
        access_token,
        refresh_token,
    })
}

pub async fn decode_access_token(token: &str, redis_store: &RedisStore) -> Result<AccessClaims, JwtError> {
    dotenv().ok();
    let secret = env::var("SECRET_KEY").expect("SECRET_KEY must be set");
    
    // First validate token in Redis
    let user_id = redis_store.validate_token(token)
        .await
        .map_err(|_| JwtError::from(jsonwebtoken::errors::ErrorKind::InvalidToken))?
        .ok_or_else(|| JwtError::from(jsonwebtoken::errors::ErrorKind::InvalidToken))?;
    
    // Then decode JWT
    let token_data = decode::<AccessClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default()
    )?;
    
    // Verify the user_id matches and it's an access token
    if token_data.claims.sub != user_id || token_data.claims.token_type != "access" {
        return Err(JwtError::from(jsonwebtoken::errors::ErrorKind::InvalidToken));
    }
    
    Ok(token_data.claims)
}

pub async fn decode_refresh_token(token: &str, redis_store: &RedisStore) -> Result<RefreshClaims, JwtError> {
    dotenv().ok();
    let secret = env::var("SECRET_KEY").expect("SECRET_KEY must be set");
    
    // First validate refresh token in Redis
    let user_id = redis_store.validate_refresh_token(token)
        .await
        .map_err(|_| JwtError::from(jsonwebtoken::errors::ErrorKind::InvalidToken))?
        .ok_or_else(|| JwtError::from(jsonwebtoken::errors::ErrorKind::InvalidToken))?;
    
    // Then decode JWT
    let token_data = decode::<RefreshClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default()
    )?;
    
    // Verify the user_id matches and it's a refresh token
    if token_data.claims.sub != user_id || token_data.claims.token_type != "refresh" {
        return Err(JwtError::from(jsonwebtoken::errors::ErrorKind::InvalidToken));
    }
    
    Ok(token_data.claims)
}

pub async fn refresh_access_token(refresh_token: &str, redis_store: &RedisStore) -> Result<String, JwtError> {
    // Decode and validate the refresh token
    let claims = decode_refresh_token(refresh_token, redis_store).await?;
    
    // Create new access token
    let access_claims = AccessClaims::new(claims.sub);
    
    dotenv().ok();
    let secret = env::var("SECRET_KEY").expect("SECRET_KEY must be set");

    let new_access_token = encode(
        &Header::default(),
        &access_claims,
        &EncodingKey::from_secret(secret.as_bytes())
    )?;

    // Store new access token in Redis
    redis_store.store_token(claims.sub, &new_access_token, access_claims.exp)
        .await
        .map_err(|_| JwtError::from(jsonwebtoken::errors::ErrorKind::InvalidToken))?;

    Ok(new_access_token)
}

pub async fn invalidate_tokens(access_token: &str, refresh_token: &str, redis_store: &RedisStore) -> Result<(), JwtError> {
    redis_store.invalidate_token(access_token)
        .await
        .map_err(|_| JwtError::from(jsonwebtoken::errors::ErrorKind::InvalidToken))?;
    
    redis_store.invalidate_refresh_token(refresh_token)
        .await
        .map_err(|_| JwtError::from(jsonwebtoken::errors::ErrorKind::InvalidToken))?;
    
    Ok(())
} 