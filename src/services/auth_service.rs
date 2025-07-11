use sqlx::SqlitePool;
use bcrypt::verify;
use tracing::{info, warn, error, instrument};

use crate::models::user::User;
use crate::models::jwt::TokenPair;
use crate::services::jwt_service::JwtService;

#[derive(Clone)]
pub struct AuthService {
    pool: SqlitePool,
    jwt_service: JwtService,
}

#[derive(Debug)]
pub enum AuthError {
    InvalidCredentials,
    DatabaseError(sqlx::Error),
    PasswordHashError,
    TokenError,
    InvalidToken,
    UserNotFound,
}

impl From<sqlx::Error> for AuthError {
    fn from(err: sqlx::Error) -> Self {
        AuthError::DatabaseError(err)
    }
}

impl From<jsonwebtoken::errors::Error> for AuthError {
    fn from(_: jsonwebtoken::errors::Error) -> Self {
        AuthError::TokenError
    }
}

impl AuthService {
    pub fn new(pool: SqlitePool, jwt_service: JwtService) -> Self {
        Self { 
            pool,
            jwt_service,
        }
    }

    #[instrument(skip(self, password))]
    pub async fn login(&self, email: &str, password: &str) -> Result<TokenPair, AuthError> {
        info!(email = %email, "Login attempt");
        
        // Find user by email
        let user = match User::find_by_email(&self.pool, email).await {
            Ok(Some(user)) => user,
            Ok(None) => {
                warn!(email = %email, "Login attempt with non-existent email");
                return Err(AuthError::InvalidCredentials);
            }
            Err(e) => {
                error!(error = %e, "Database error during login");
                return Err(AuthError::DatabaseError(e));
            }
        };

        // Verify password
        let password_matches = match verify(password, &user.password_hash) {
            Ok(matches) => matches,
            Err(e) => {
                error!(error = %e, "Password verification error");
                return Err(AuthError::PasswordHashError);
            }
        };

        if !password_matches {
            warn!(email = %email, "Failed login attempt - invalid password");
            return Err(AuthError::InvalidCredentials);
        }

        // Generate JWT tokens
        match self.jwt_service.create_tokens(user.id).await {
            Ok(token_pair) => {
                info!(user_id = %user.id, email = %email, "User successfully logged in");
                Ok(token_pair)
            }
            Err(e) => {
                error!(error = %e, "Token generation error");
                Err(AuthError::TokenError)
            }
        }
    }

    #[instrument(skip(self, password))]
    pub async fn register(
        &self,
        username: &str,
        password: &str,
        email: &str,
    ) -> Result<i64, AuthError> {
        info!(username = %username, email = %email, "New user registration attempt");

        // Check if user already exists
        if let Ok(Some(_)) = User::find_by_email(&self.pool, email).await {
            warn!(email = %email, "Registration attempt with existing email");
            return Err(AuthError::InvalidCredentials);
        }

        match User::create(&self.pool, username, password, email).await {
            Ok(user) => {
                info!(
                    user_id = %user.id,
                    username = %username,
                    email = %email,
                    "New user successfully registered"
                );
                Ok(user.id)
            }
            Err(e) => {
                error!(error = %e, "Failed to create new user");
                Err(AuthError::DatabaseError(e))
            }
        }
    }

} 