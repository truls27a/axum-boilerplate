use axum::{
    Json,
    http::{StatusCode, HeaderMap},
    extract::State,
};
use serde::{Deserialize, Serialize};

use crate::services::auth_service::{AuthService, AuthError};
use crate::utils::cookies::CookieManager;
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
pub struct TokenResponse {
    access_token: String,
}

#[derive(Serialize)]
pub struct RefreshTokenResponse {
    access_token: String,
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<(HeaderMap, Json<TokenResponse>), StatusCode> {
    let auth_service = AuthService::new(state.db, state.jwt_service);
    
    let token_pair = auth_service
        .login(&payload.email, &payload.password)
        .await
        .map_err(|e| match e {
            AuthError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })?;

    let headers = CookieManager::create_refresh_token_cookie(&token_pair.refresh_token);

    Ok((headers, Json(TokenResponse { 
        access_token: token_pair.access_token,
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
    if let Some(token) = CookieManager::extract_refresh_token(&headers) {
        // Invalidate the refresh token using jwt_service
        if let Err(_) = state.jwt_service.invalidate_tokens("", &token).await {
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    Ok(CookieManager::clear_refresh_token_cookie())
}
