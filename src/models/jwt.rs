use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};

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

#[derive(Debug, Serialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
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