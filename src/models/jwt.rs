// src/models/jwt.rs
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessClaims {
    pub sub: i64,          // user id
    pub exp: i64,          // expiration time
    pub iat: i64,          // issued at
    pub token_type: String // "access"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshClaims {
    pub sub: i64,          // user id
    pub exp: i64,          // expiration time
    pub iat: i64,          // issued at
    pub jti: String,       // unique id for allow/deny list
    pub token_type: String // "refresh"
}

#[derive(Debug, Serialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
}

impl AccessClaims {
    pub fn new(user_id: i64) -> Self {
        let now = Utc::now();
        let expires_at = now + Duration::minutes(15);

        Self {
            sub: user_id,
            exp: expires_at.timestamp(),
            iat: now.timestamp(),
            token_type: "access".to_string(),
        }
    }
}

impl RefreshClaims {
    pub fn new(user_id: i64, jti: String) -> Self {
        let now = Utc::now();
        let expires_at = now + Duration::days(7);

        Self {
            sub: user_id,
            exp: expires_at.timestamp(),
            iat: now.timestamp(),
            jti,
            token_type: "refresh".to_string(),
        }
    }
}
