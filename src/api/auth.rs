use axum::{
    Json,
    http::StatusCode,
    extract::State,
};
use serde::{Deserialize, Serialize};

use crate::services::auth_service::{AuthService, AuthError};
use crate::AppState;

#[derive(Deserialize)]
pub struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    username: String,
    password: String,
    email: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    token: String,
}

#[derive(Serialize)]
pub struct RegisterResponse {
    id: i64,
}

#[derive(Serialize)]
pub struct TokenResponse {
    access_token: String,
    refresh_token: String,
}

#[derive(Serialize)]
pub struct RefreshTokenResponse {
    access_token: String,
}

#[derive(Deserialize)]
pub struct RefreshTokenRequest {
    refresh_token: String,
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<TokenResponse>, StatusCode> {
    let auth_service = AuthService::new(state.db);
    
    let token_pair = auth_service
        .login(&payload.email, &payload.password, &state.redis)
        .await
        .map_err(|e| match e {
            AuthError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })?;

    Ok(Json(TokenResponse { 
        access_token: token_pair.access_token,
        refresh_token: token_pair.refresh_token,
    }))
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, StatusCode> {
    let auth_service = AuthService::new(state.db);
    
    let id = auth_service
        .register(&payload.username, &payload.password, &payload.email)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(RegisterResponse { id }))
}

pub async fn refresh_token(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<Json<RefreshTokenResponse>, StatusCode> {
    let auth_service = AuthService::new(state.db);
    
    let new_access_token = auth_service
        .refresh_token(&payload.refresh_token, &state.redis)
        .await
        .map_err(|e| match e {
            AuthError::InvalidToken => StatusCode::UNAUTHORIZED,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })?;

    Ok(Json(RefreshTokenResponse { 
        access_token: new_access_token 
    }))
}
