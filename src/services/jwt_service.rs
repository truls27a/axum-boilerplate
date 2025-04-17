use jsonwebtoken::{encode, decode, EncodingKey, DecodingKey, Header, Validation, errors::Error as JwtError};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};
use std::env;

use crate::db::RedisStore;
use crate::models::jwt::{AccessClaims, RefreshClaims, TokenPair};

#[derive(Clone)]
pub struct JwtService {
    redis_store: RedisStore,
    secret_key: String,
}

impl JwtService {
    pub fn new(redis_store: RedisStore, secret_key: String) -> Self {
        Self {
            redis_store,
            secret_key,
        }
    }

    pub async fn create_token_pair(&self, user_id: i64) -> Result<TokenPair, JwtError> {
        let access_claims = AccessClaims::new(user_id);
        let refresh_claims = RefreshClaims::new(user_id);

        let access_token = encode(
            &Header::default(),
            &access_claims,
            &EncodingKey::from_secret(self.secret_key.as_bytes())
        )?;

        let refresh_token = encode(
            &Header::default(),
            &refresh_claims,
            &EncodingKey::from_secret(self.secret_key.as_bytes())
        )?;

        // Store both tokens in Redis
        self.redis_store.store_token(user_id, &access_token, access_claims.exp)
            .await
            .map_err(|_| JwtError::from(jsonwebtoken::errors::ErrorKind::InvalidToken))?;

        self.redis_store.store_refresh_token(user_id, &refresh_token, refresh_claims.exp)
            .await
            .map_err(|_| JwtError::from(jsonwebtoken::errors::ErrorKind::InvalidToken))?;

        Ok(TokenPair {
            access_token,
            refresh_token,
        })
    }

    pub async fn decode_access_token(&self, token: &str) -> Result<AccessClaims, JwtError> {
        // First validate token in Redis
        let user_id = self.redis_store.validate_token(token)
            .await
            .map_err(|_| JwtError::from(jsonwebtoken::errors::ErrorKind::InvalidToken))?
            .ok_or_else(|| JwtError::from(jsonwebtoken::errors::ErrorKind::InvalidToken))?;
        
        // Then decode JWT
        let token_data = decode::<AccessClaims>(
            token,
            &DecodingKey::from_secret(self.secret_key.as_bytes()),
            &Validation::default()
        )?;
        
        // Verify the user_id matches and it's an access token
        if token_data.claims.sub != user_id || token_data.claims.token_type != "access" {
            return Err(JwtError::from(jsonwebtoken::errors::ErrorKind::InvalidToken));
        }
        
        Ok(token_data.claims)
    }

    pub async fn decode_refresh_token(&self, token: &str) -> Result<RefreshClaims, JwtError> {
        // First validate refresh token in Redis
        let user_id = self.redis_store.validate_refresh_token(token)
            .await
            .map_err(|_| JwtError::from(jsonwebtoken::errors::ErrorKind::InvalidToken))?
            .ok_or_else(|| JwtError::from(jsonwebtoken::errors::ErrorKind::InvalidToken))?;
        
        // Then decode JWT
        let token_data = decode::<RefreshClaims>(
            token,
            &DecodingKey::from_secret(self.secret_key.as_bytes()),
            &Validation::default()
        )?;
        
        // Verify the user_id matches and it's a refresh token
        if token_data.claims.sub != user_id || token_data.claims.token_type != "refresh" {
            return Err(JwtError::from(jsonwebtoken::errors::ErrorKind::InvalidToken));
        }
        
        Ok(token_data.claims)
    }

    pub async fn refresh_access_token(&self, refresh_token: &str) -> Result<String, JwtError> {
        // Decode and validate the refresh token
        let claims = self.decode_refresh_token(refresh_token).await?;
        
        // Create new access token
        let access_claims = AccessClaims::new(claims.sub);
        
        let new_access_token = encode(
            &Header::default(),
            &access_claims,
            &EncodingKey::from_secret(self.secret_key.as_bytes())
        )?;

        // Store new access token in Redis
        self.redis_store.store_token(claims.sub, &new_access_token, access_claims.exp)
            .await
            .map_err(|_| JwtError::from(jsonwebtoken::errors::ErrorKind::InvalidToken))?;

        Ok(new_access_token)
    }

    pub async fn invalidate_tokens(&self, access_token: &str, refresh_token: &str) -> Result<(), JwtError> {
        self.redis_store.invalidate_token(access_token)
            .await
            .map_err(|_| JwtError::from(jsonwebtoken::errors::ErrorKind::InvalidToken))?;
        
        self.redis_store.invalidate_refresh_token(refresh_token)
            .await
            .map_err(|_| JwtError::from(jsonwebtoken::errors::ErrorKind::InvalidToken))?;
        
        Ok(())
    }
} 