use axum::{
    Json,
    http::StatusCode,
    extract::State,
};
use serde::{Deserialize, Serialize};
use bcrypt::{verify};
use sqlx::SqlitePool;

use crate::models::user::User;
use crate::utils::jwt;

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

pub async fn login(
    State(pool): State<SqlitePool>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    // Find user by username
    let user = User::find_by_email(&pool, &payload.email)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Verify password
    let password_matches = verify(&payload.password, &user.password_hash)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !password_matches {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Generate JWT token
    let token = jwt::create_token(user.id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(LoginResponse { token }))
}

pub async fn register(
    State(pool): State<SqlitePool>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, StatusCode> {
    // Create user
    let user = User::create(&pool, &payload.username, &payload.password, &payload.email)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(RegisterResponse { id: user.id }))
}
