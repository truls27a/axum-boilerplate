use jsonwebtoken::{encode, EncodingKey, Header, errors::Error as JwtError};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};

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

pub fn create_token(user_id: i64) -> Result<String, JwtError> {
    let claims = Claims::new(user_id);
    // In production, use a proper secret key from environment variables
    let secret = b"your_secret_key";
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret))
} 