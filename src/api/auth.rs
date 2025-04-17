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
    username: String,
    password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    token: String,
}

pub async fn login(
    State(pool): State<SqlitePool>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    // Find user by username
    let user = User::find_by_username(&pool, &payload.username)
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