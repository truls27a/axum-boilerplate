use axum::{
    Json,
    http::{StatusCode, HeaderMap},
    extract::State,
};
use serde::{Deserialize, Serialize};
use tracing::{info, error, debug};

use crate::services::auth_service::AuthError;
use crate::AppState;
use jsonwebtoken::errors::ErrorKind;
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
    message: String,
    success: bool,
    id: i64,
}

#[derive(Serialize)]
pub struct LoginResponse {
    message: String,
    success: bool,
}

#[derive(Serialize)]
pub struct RefreshTokenResponse {
    message: String,
    success: bool,
}

#[derive(Serialize)]
pub struct LogoutResponse {
    message: String,
    success: bool,
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<(HeaderMap, Json<LoginResponse>), StatusCode> {
    debug!("Login attempt for email: {}", payload.email);
    
    let token_pair = state.auth_service
        .login(&payload.email, &payload.password)
        .await
        .map_err(|e| match e {
            AuthError::InvalidCredentials => {
                error!("Invalid credentials for email: {}", payload.email);
                StatusCode::UNAUTHORIZED
            },
            _ => {
                error!("Internal server error during login: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    // Set auth cookies
    let headers = CookieService::set_auth_cookies(&token_pair.access_token, &token_pair.refresh_token);
    info!("User successfully logged in: {}", payload.email);

    Ok((headers, Json(LoginResponse { 
        message: "Login successful".to_string(),
        success: true,
    })))
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, StatusCode> {
    debug!("Registration attempt for email: {}", payload.email);

    let id = state.auth_service
        .register(&payload.username, &payload.password, &payload.email)
        .await
        .map_err(|e| {
            error!("Failed to register user: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!("Successfully registered new user with id: {}", id);
    Ok(Json(RegisterResponse { 
        message: "Registration successful".to_string(),
        success: true,
        id,
    }))
}

pub async fn refresh_token(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<(HeaderMap, Json<RefreshTokenResponse>), StatusCode> {
    debug!("Token refresh attempt");

    let refresh_token = CookieService::extract_token(&headers, REFRESH_TOKEN_COOKIE)
        .ok_or_else(|| {
            error!("No refresh token found in request");
            StatusCode::UNAUTHORIZED
        })?;

    let new_access_token = state.jwt_service
        .refresh_tokens(&refresh_token)
        .await
        .map_err(|e| match e.kind() {
            ErrorKind::InvalidToken => {
                error!("Invalid refresh token provided");
                StatusCode::UNAUTHORIZED
            },
            _ => {
                error!("Internal server error during token refresh: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    // Set auth cookies
    let headers = CookieService::set_auth_cookies(&new_access_token.access_token, &new_access_token.refresh_token);
    info!("Successfully refreshed tokens");

    Ok((headers, Json(RefreshTokenResponse { 
        message: "Tokens refreshed successfully".to_string(),
        success: true,
    })))
}

pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<(HeaderMap, Json<LogoutResponse>), StatusCode> {
    debug!("Logout attempt");

    let refresh_token = CookieService::extract_token(&headers, REFRESH_TOKEN_COOKIE)
        .ok_or_else(|| {
            error!("No refresh token found during logout");
            StatusCode::UNAUTHORIZED
        })?;
    
    // Revoke the refresh token
    state.jwt_service
        .revoke_token(&refresh_token)
        .await
        .map_err(|e| {
            error!("Failed to revoke refresh token: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Clear auth cookies
    let headers = CookieService::clear_auth_cookies();
    info!("User successfully logged out");
    
    Ok((headers, Json(LogoutResponse { 
        message: "Logout successful".to_string(),
        success: true,
    })))
}
