use axum::{
    extract::Extension,
    response::IntoResponse,
    Json,
};
use serde::Serialize;

use crate::middleware::auth::CurrentUser;

#[derive(Serialize)]
pub struct UserResponse {
    id: i64,
    username: String,
    email: String,
}

pub async fn get_current_user(
    Extension(current_user): Extension<CurrentUser>,
) -> impl IntoResponse {
    Json(UserResponse {
        id: current_user.0.id,
        username: current_user.0.username,
        email: current_user.0.email,
    })
} 