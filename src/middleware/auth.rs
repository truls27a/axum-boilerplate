use axum::{
    async_trait,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
    body::Body,
};

use crate::{
    models::user::User,
    services::auth_service::AuthService,
    AppState,
};

#[derive(Clone)]
pub struct CurrentUser(pub User);

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Get the authorization header
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|header| header.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Check if it starts with "Bearer "
    if !auth_header.starts_with("Bearer ") {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Extract the token
    let token = &auth_header["Bearer ".len()..];
    
    // Verify the token and get the user
    let auth_service = AuthService::new(state.db, state.jwt_service);
    let user = auth_service
        .verify_token(token)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Add the user to request extensions
    request.extensions_mut().insert(CurrentUser(user));

    // Continue with the request
    Ok(next.run(request).await)
} 