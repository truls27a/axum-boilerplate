#[cfg(test)]
use axum::{
    http::{Request, StatusCode, HeaderMap},
    body::Body,
};
use serde_json::{json, Value};
use super::helpers::{setup_test_db, create_test_app, test_request, extract_response_cookie};
use tower::ServiceExt;
use crate::services::cookie_service::{CookieService, ACCESS_TOKEN_COOKIE, REFRESH_TOKEN_COOKIE};
use tracing::{info, debug};
use tower_cookies::Cookie;

#[tokio::test]
async fn test_register_success() {
    let pool = setup_test_db().await;
    let app = create_test_app(pool);

    let username = "testuser";
    let email = "test@example.com";
    let password = "password123";

    let register_data = json!({
        "username": username,
        "email": email,
        "password": password
    });

    let (status, body, _) = test_request(
        app,
        "POST",
        "/register",
        Some(register_data),
        None,
        None,
    ).await;

    assert_eq!(status, StatusCode::OK);
    let response: Value = serde_json::from_str(&body).unwrap();
    assert!(response.get("id").is_some());
}

#[tokio::test]
async fn test_login_success() {
    // Setup
    let pool = setup_test_db().await;
    let app = create_test_app(pool.clone());

    let username = "testuser";
    let email = "test@example.com";
    let password = "password123";

    // First register a user
    let register_data = json!({
        "username": username,
        "email": email,
        "password": password
    });

    let (status, _, _) = test_request(
        app.clone(),
        "POST",
        "/register",
        Some(register_data),
        None,
        None,
    ).await;

    assert_eq!(status, StatusCode::OK);

    // Now test login
    let login_data = json!({
        "email": email,
        "password": password,
    });

    let (status, body, headers) = test_request(
        app,
        "POST",
        "/login",
        Some(login_data),
        None,
        None,
    ).await;

    assert_eq!(status, StatusCode::OK);
    
    // Parse response and verify message
    let response: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(response["message"], "Login successful");
    
    // Extract tokens from Set-Cookie headers
    let access_token = extract_response_cookie(&headers, ACCESS_TOKEN_COOKIE);
    let refresh_token = extract_response_cookie(&headers, REFRESH_TOKEN_COOKIE);
    
    assert!(access_token.is_some());
    assert!(refresh_token.is_some());
}

#[tokio::test]
async fn test_refresh_token_success() {
    // Setup and login
    let pool = setup_test_db().await;
    let app = create_test_app(pool.clone());

    // Register and login a user
    let email = "test@example.com";
    let password = "password123";
    
    let register_data = json!({
        "username": "testuser",
        "email": email,
        "password": password
    });

    debug!("Registering test user");
    let (status, _, _) = test_request(
        app.clone(),
        "POST",
        "/register",
        Some(register_data),
        None,
        None,
    ).await;
    assert_eq!(status, StatusCode::OK);

    let login_data = json!({
        "email": email,
        "password": password,
    });

    debug!("Logging in test user");
    let (status, _, headers) = test_request(
        app.clone(),
        "POST",
        "/login",
        Some(login_data),
        None,
        None,
    ).await;
    assert_eq!(status, StatusCode::OK);

    // Add a small delay to ensure different timestamps
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // Extract tokens from Set-Cookie headers
    let access_token = extract_response_cookie(&headers, ACCESS_TOKEN_COOKIE).unwrap();
    let refresh_token = extract_response_cookie(&headers, REFRESH_TOKEN_COOKIE).unwrap();
    let cookies = vec![(ACCESS_TOKEN_COOKIE, access_token.as_str()), (REFRESH_TOKEN_COOKIE, refresh_token.as_str())];

    debug!("Cookies: {:?}", cookies);

    // Test refresh token endpoint
    debug!("Attempting to refresh token");
    let (status, body, headers) = test_request(
        app,
        "POST",
        "/refresh",
        None,
        None,
        Some(&cookies),
    ).await;

    debug!(status = %status, "Refresh token response status");
    if status != StatusCode::OK {
        let body_str = String::from_utf8_lossy(body.as_bytes());
        debug!(response_body = %body_str, "Error response body");
    }

    assert_eq!(status, StatusCode::OK);
    let response: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(response["message"], "Tokens refreshed successfully");

    // Verify new cookies are present
    let new_access_token = extract_response_cookie(&headers, ACCESS_TOKEN_COOKIE);
    let new_refresh_token = extract_response_cookie(&headers, REFRESH_TOKEN_COOKIE);
    
    assert!(new_access_token.is_some());
    assert!(new_refresh_token.is_some());
    assert_ne!(new_access_token.unwrap(), access_token);
    assert_ne!(new_refresh_token.unwrap(), refresh_token);
}

#[tokio::test]
async fn test_refresh_token_invalid() {
    let pool = setup_test_db().await;
    let app = create_test_app(pool);

    // Try to refresh with invalid token
    let invalid_cookies = vec![(REFRESH_TOKEN_COOKIE, "invalid_token")];

    let (status, _, _) = test_request(
        app,
        "POST",
        "/refresh",
        None,
        None,
        Some(&invalid_cookies),
    ).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_logout_success() {
    // Setup and login
    let pool = setup_test_db().await;
    let app = create_test_app(pool.clone());

    // Register and login a user
    let email = "test@example.com";
    let password = "password123";
    
    let register_data = json!({
        "username": "testuser",
        "email": email,
        "password": password
    });

    let (_, _, _) = test_request(
        app.clone(),
        "POST",
        "/register",
        Some(register_data),
        None,
        None,
    ).await;

    let login_data = json!({
        "email": email,
        "password": password,
    });

    let (_, _, headers) = test_request(
        app.clone(),
        "POST",
        "/login",
        Some(login_data),
        None,
        None,
    ).await;

    // Extract tokens from Set-Cookie headers
    let access_token = extract_response_cookie(&headers, ACCESS_TOKEN_COOKIE).unwrap();
    let refresh_token = extract_response_cookie(&headers, REFRESH_TOKEN_COOKIE).unwrap();
    let cookies = vec![(ACCESS_TOKEN_COOKIE, access_token.as_str()), (REFRESH_TOKEN_COOKIE, refresh_token.as_str())];

    debug!("Cookies before logout request: {:?}", cookies);
    debug!("Refresh token value: {}", refresh_token);

    // Test logout endpoint
    let (status, body, headers) = test_request(
        app.clone(),
        "POST",
        "/logout",
        None,
        None,
        Some(&cookies),
    ).await;

    debug!("Logout response status: {}", status);
    if status != StatusCode::OK {
        debug!("Logout response body: {}", body);
        debug!("Logout response headers: {:?}", headers);
    }

    assert_eq!(status, StatusCode::OK);

    // Verify that both cookies are cleared by trying to use them
    let (status, _, _) = test_request(
        app,
        "POST",
        "/refresh",
        None,
        None,
        Some(&cookies),
    ).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED, "Cookies should be invalid after logout");
}

#[tokio::test]
async fn test_login_invalid_password() {
    let pool = setup_test_db().await;
    let app = create_test_app(pool.clone());

    // Register user
    let register_data = json!({
        "username": "testuser",
        "email": "test@example.com",
        "password": "password123"
    });

    let (_, _, _) = test_request(
        app.clone(),
        "POST",
        "/register",
        Some(register_data),
        None,
        None,
    ).await;

    // Test login with wrong password
    let login_data = json!({
        "email": "test@example.com",
        "password": "wrongpassword",
    });

    let (status, _, _) = test_request(
        app,
        "POST",
        "/login",
        Some(login_data),
        None,
        None,
    ).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_get_current_user() {
    let pool = setup_test_db().await;
    let app = create_test_app(pool.clone());

    let username = "testuser";
    let email = "test@example.com";
    let password = "password123";

    // Register user
    let register_data = json!({
        "username": username,
        "email": email,
        "password": password
    });

    let (_, _, _) = test_request(
        app.clone(),
        "POST",
        "/register",
        Some(register_data),
        None,
        None,
    ).await;

    // Login to get cookies
    let login_data = json!({
        "email": email,
        "password": password,
    });

    let (_, _, headers) = test_request(
        app.clone(),
        "POST",
        "/login",
        Some(login_data),
        None,
        None,
    ).await;

    // Extract tokens from cookies
    let cookies: Vec<_> = headers.get_all("set-cookie")
        .iter()
        .filter_map(|c| c.to_str().ok())
        .map(|c| {
            let cookie = Cookie::parse(c).unwrap();
            (cookie.name().to_string(), cookie.value().to_string())
        })
        .collect();

    // Test /me endpoint with cookies
    let (status, body, _) = test_request(
        app,
        "GET",
        "/me",
        None,
        None,
        Some(&cookies.iter().map(|(n, v)| (n.as_str(), v.as_str())).collect::<Vec<_>>()),
    ).await;

    let user_response: Value = serde_json::from_str(&body).unwrap();

    assert_eq!(status, StatusCode::OK);
    assert_eq!(user_response["username"], username);
    assert_eq!(user_response["email"], email);
    assert!(user_response["id"].is_number());
} 