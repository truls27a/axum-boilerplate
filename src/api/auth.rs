use axum::{
    Json,
    http::{StatusCode, HeaderMap},
    extract::State,
};
use serde::{Deserialize, Serialize};

use crate::services::auth_service::{AuthError};
use crate::AppState;
use jsonwebtoken::{
    errors::Error as JwtError, errors::ErrorKind
};
use crate::services::cookie_service::{CookieService, REFRESH_TOKEN_COOKIE};

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
    message: String,
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<(HeaderMap, Json<LoginResponse>), StatusCode> {
    
    let token_pair = state.auth_service
        .login(&payload.email, &payload.password)
        .await
        .map_err(|e| match e {
            AuthError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })?;

    // Set auth cookies
    let headers = CookieService::set_auth_cookies(&token_pair.access_token, &token_pair.refresh_token);

    Ok((headers, Json(LoginResponse { 
        message: "Login successful".to_string(),
    })))
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, StatusCode> {
    let id = state.auth_service
        .register(&payload.username, &payload.password, &payload.email)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(RegisterResponse { id }))
}

pub async fn refresh_token(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<(HeaderMap, Json<RefreshTokenResponse>), StatusCode> {
    let refresh_token = CookieService::extract_token(&headers, REFRESH_TOKEN_COOKIE)
        .ok_or(StatusCode::UNAUTHORIZED)?;

    
    let new_access_token = state.jwt_service
        .refresh_tokens(&refresh_token)
        .await
        .map_err(|e| match e.kind() {
            ErrorKind::InvalidToken => StatusCode::UNAUTHORIZED,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })?;

    // Set auth cookies
    let headers = CookieService::set_auth_cookies(&new_access_token.access_token, &new_access_token.refresh_token);

    Ok((headers, Json(RefreshTokenResponse { 
        message: "Tokens refreshed successfully".to_string(),
    })))
}

pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<HeaderMap, StatusCode> {
    let refresh_token = CookieService::extract_token(&headers, REFRESH_TOKEN_COOKIE)
        .ok_or(StatusCode::UNAUTHORIZED)?;

    
    // Revoke the refresh token
    state.jwt_service
        .revoke_token(&refresh_token)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Clear auth cookies
    let headers = CookieService::clear_auth_cookies();
    
    Ok(headers)
}
