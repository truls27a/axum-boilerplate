// src/services/jwt_service.rs
use crate::db::RedisStore;
use crate::models::jwt::{AccessClaims, RefreshClaims, TokenPair};

use chrono::Utc;
use jsonwebtoken::{
    decode, encode, errors::Error as JwtError, errors::ErrorKind, Algorithm, DecodingKey, EncodingKey, Header,
    Validation,
};
use tracing::{error, info, instrument};
use uuid::Uuid;

#[derive(Clone)]
pub struct JwtService {
    redis_store: RedisStore,
    secret_key: String,
    enc_key: EncodingKey,
    dec_key: DecodingKey,
}

impl JwtService {
    pub fn new(redis_store: RedisStore, secret_key: String) -> Self {
        let enc_key = EncodingKey::from_secret(secret_key.as_bytes());
        let dec_key = DecodingKey::from_secret(secret_key.as_bytes());

        Self {
            redis_store,
            secret_key,
            enc_key,
            dec_key,
        }
    }

    /* ---------- PUBLIC API ---------- */

    /// Generate and allow-list a fresh token pair.
    #[instrument(skip(self))]
    pub async fn create_tokens(&self, user_id: i64) -> Result<TokenPair, JwtError> {
        // Generate a unique ID for the refresh token
        let refresh_jti = Uuid::new_v4().to_string();

        let access_token = self.create_jwt(&AccessClaims::new(user_id))?;
        let refresh_claims = RefreshClaims::new(user_id, refresh_jti.clone());
        let refresh_token = self.create_jwt(&refresh_claims)?;

        // put refresh JTI into allow-list
        let ttl = (refresh_claims.exp - refresh_claims.iat) as usize;
        if let Err(e) = self
            .redis_store
            .add_to_allowlist(&refresh_jti, user_id, ttl)
            .await
        {
            error!(error = %e, "Failed to allow-list refresh token");
        }

        Ok(TokenPair {
            access_token,
            refresh_token,
        })
    }

    /// Validate an access token and return its claims.
    /// Fails if expired or black-listed.
    #[instrument(skip(self))]
    pub async fn verify_access_token(&self, token: &str) -> Result<AccessClaims, JwtError> {
        if self.redis_store.is_blacklisted(token).await.unwrap_or(false) {
            return Err(ErrorKind::ExpiredSignature.into());
        }
        let data = self.decode_jwt::<AccessClaims>(token)?;

        Ok(data)
    }

    /// Exchange a valid refresh token for a brand-new pair.
    ///  1. Must still be allow-listed
    ///  2. Not black-listed / expired
    ///  3. Old refresh token is revoked
    #[instrument(skip(self))]
    pub async fn refresh_tokens(&self, refresh_token: &str) -> Result<TokenPair, JwtError> {
        if self
            .redis_store
            .is_blacklisted(refresh_token)
            .await
            .unwrap_or(false)
        {
            return Err(ErrorKind::InvalidToken.into());
        }

        let claims = self.decode_jwt::<RefreshClaims>(refresh_token)?;

        // ensure still allow-listed
        if !self
            .redis_store
            .is_allowlisted(&claims.jti)
            .await
            .unwrap_or(false)
        {
            return Err(ErrorKind::InvalidToken.into());
        }

        // everything checks out ⇒ revoke old refresh & build new pair
        self.revoke_token(refresh_token, claims.exp - Utc::now().timestamp())
            .await;

        self.redis_store
            .remove_from_allowlist(&claims.jti)
            .await
            .ok();

        self.create_tokens(claims.sub).await
    }

    /// Revoke a token (access *or* refresh) immediately.
    #[instrument(skip(self))]
    pub async fn revoke_token(&self, token: &str, ttl_seconds: i64) {
        if ttl_seconds <= 0 {
            return;
        }
        if let Err(e) = self
            .redis_store
            .blacklist_token(token, ttl_seconds as usize)
            .await
        {
            error!(error = %e, "Failed to blacklist token");
        }
    }

    /* ---------- PRIVATE HELPERS ---------- */

    fn create_jwt<T: serde::Serialize>(&self, claims: &T) -> Result<String, JwtError> {
        encode(&Header::default(), claims, &self.enc_key)
    }

    fn decode_jwt<T: serde::de::DeserializeOwned>(&self, token: &str) -> Result<T, JwtError> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;

        decode::<T>(token, &self.dec_key, &validation).map(|data| data.claims)
    }
}
