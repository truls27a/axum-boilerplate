use sqlx::SqlitePool;
use bcrypt::verify;

use crate::models::user::User;
use crate::utils::jwt;

#[derive(Clone)]
pub struct AuthService {
    pool: SqlitePool,
}

#[derive(Debug)]
pub enum AuthError {
    InvalidCredentials,
    DatabaseError(sqlx::Error),
    PasswordHashError,
    TokenError,
}

impl From<sqlx::Error> for AuthError {
    fn from(err: sqlx::Error) -> Self {
        AuthError::DatabaseError(err)
    }
}

impl AuthService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<String, AuthError> {
        // Find user by email
        let user = User::find_by_email(&self.pool, email)
            .await
            .map_err(AuthError::from)?
            .ok_or(AuthError::InvalidCredentials)?;

        // Verify password
        let password_matches = verify(password, &user.password_hash)
            .map_err(|_| AuthError::PasswordHashError)?;

        if !password_matches {
            return Err(AuthError::InvalidCredentials);
        }

        // Generate JWT token
        let token = jwt::create_token(user.id)
            .map_err(|_| AuthError::TokenError)?;

        Ok(token)
    }

    pub async fn register(
        &self,
        username: &str,
        password: &str,
        email: &str,
    ) -> Result<i64, AuthError> {
        let user = User::create(&self.pool, username, password, email)
            .await
            .map_err(AuthError::from)?;

        Ok(user.id)
    }
} 