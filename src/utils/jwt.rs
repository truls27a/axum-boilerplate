use jsonwebtoken::{encode, decode, EncodingKey, DecodingKey, Header, Validation, errors::Error as JwtError};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};
use dotenv::dotenv;
use std::env;

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

    // Load .env file
    dotenv().ok();

    let secret = env::var("SECRET_KEY").expect("SECRET_KEY must be set");

    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
}

pub fn decode_token(token: &str) -> Result<Claims, JwtError> {
    dotenv().ok();
    let secret = env::var("SECRET_KEY").expect("SECRET_KEY must be set");
    
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default()
    ).map(|data| data.claims)
} 