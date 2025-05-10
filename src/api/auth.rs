use axum::{
    Json,
    http::{StatusCode, HeaderMap},
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
pub struct RegisterResponse {
    id: i64,
}

#[derive(Serialize)]
pub struct LoginResponse {
    message: String,
}

#[derive(Serialize)]
pub struct RefreshTokenResponse {
    access_token: String,
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<(HeaderMap, Json<LoginResponse>), StatusCode> {
    let auth_service = AuthService::new(state.db, state.jwt_service);
    
    let token_pair = auth_service
        .login(&payload.email, &payload.password)
        .await
        .map_err(|e| match e {
            AuthError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })?;


    // Create tokens and set them in cookies

    Ok((headers, Json(LoginResponse { 
        message: "Login successful".to_string(),
    })))
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, StatusCode> {
    let auth_service = AuthService::new(state.db, state.jwt_service);
    
    let id = auth_service
        .register(&payload.username, &payload.password, &payload.email)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(RegisterResponse { id }))
}

pub async fn refresh_token(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<(HeaderMap, Json<RefreshTokenResponse>), StatusCode> {
    let refresh_token = CookieManager::extract_refresh_token(&headers)
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let auth_service = AuthService::new(state.db, state.jwt_service);
    
    let new_access_token = auth_service
        .refresh_token(&refresh_token)
        .await
        .map_err(|e| match e {
            AuthError::InvalidToken => StatusCode::UNAUTHORIZED,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })?;

    Ok((HeaderMap::new(), Json(RefreshTokenResponse { 
        access_token: new_access_token 
    })))
}

pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<HeaderMap, StatusCode> {
    // Revoke the refresh token
    // Delete the cookies
    // Return message
}
