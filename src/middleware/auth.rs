use axum::{
    async_trait,
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use tracing::{info, warn, error, debug};

use crate::{
    AppState,
    models::user::User,
    services::{
        auth_service::AuthService,
        cookie_service::{ACCESS_TOKEN_COOKIE, CookieService},
    },
};

#[derive(Clone)]
pub struct CurrentUser(pub User);

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    debug!("Auth middleware started");
    debug!("Request headers: {:?}", request.headers());

    // Get the access token from cookies
    let access_token = match CookieService::extract_token(&request.headers(), ACCESS_TOKEN_COOKIE) {
        Some(token) => {
            debug!(token_length = token.len(), "Access token found in cookies");
            token
        },
        None => {
            warn!("No access token found in cookies");
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    // Decode and verify the token
    let claims = match state.jwt_service.verify_access_token(&access_token).await {
        Ok(claims) => {
            debug!(user_id = %claims.sub, "Token verified successfully");
            claims
        },
        Err(e) => {
            warn!(error = %e, "Invalid token verification attempt");
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    // Find user by ID
    let user = match User::find_by_id(&state.db, claims.sub).await {
        Ok(Some(user)) => {
            info!(user_id = %user.id, "User found and authenticated successfully");
            user
        }
        Ok(None) => {
            warn!(user_id = %claims.sub, "Token verification failed - user not found");
            return Err(StatusCode::UNAUTHORIZED);
        }
        Err(e) => {
            error!(error = %e, "Database error during token verification");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Add the user to request extensions
    request.extensions_mut().insert(CurrentUser(user));
    debug!("User added to request extensions");

    // Continue with the request
    Ok(next.run(request).await)
}
