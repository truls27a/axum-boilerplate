#[cfg(test)]
use axum::{
    http::{Request, StatusCode, HeaderMap},
    body::Body,
};
use serde_json::{json, Value};
use super::helpers::{setup_test_db, create_test_app, test_request};
use tower::ServiceExt;
use crate::services::cookie_service::{ACCESS_TOKEN_COOKIE, REFRESH_TOKEN_COOKIE};
use tracing::{info, debug};

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
    
    // Extract tokens from cookies
    let cookies: Vec<_> = headers.get_all("set-cookie")
        .iter()
        .filter_map(|c| c.to_str().ok())
        .map(|c| {
            let parts: Vec<_> = c.split(';').next().unwrap().split('=').collect();
            (parts[0], parts[1])
        })
        .collect();

    assert_eq!(cookies.len(), 2);
    assert!(cookies.iter().any(|(name, _)| *name == ACCESS_TOKEN_COOKIE));
    assert!(cookies.iter().any(|(name, _)| *name == REFRESH_TOKEN_COOKIE));
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

    // Extract tokens from cookies
    let cookies: Vec<_> = headers.get_all("set-cookie")
        .iter()
        .filter_map(|c| c.to_str().ok())
        .map(|c| {
            let parts: Vec<_> = c.split(';').next().unwrap().split('=').collect();
            debug!(cookie_name = %parts[0], "Extracted cookie from login response");
            (parts[0], parts[1])
        })
        .collect();

    debug!(cookie_count = cookies.len(), "Number of cookies received");
    for (name, _) in &cookies {
        debug!(cookie_name = %name, "Found cookie");
    }

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
    let new_cookies: Vec<_> = headers.get_all("set-cookie")
        .iter()
        .filter_map(|c| c.to_str().ok())
        .map(|c| {
            let parts: Vec<_> = c.split(';').next().unwrap().split('=').collect();
            debug!(cookie_name = %parts[0], "Received new cookie after refresh");
            (parts[0], parts[1])
        })
        .collect();
    debug!(new_cookie_count = new_cookies.len(), "Number of new cookies received");
    assert_eq!(new_cookies.len(), 2);
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

    // Extract tokens from cookies
    let cookies: Vec<_> = headers.get_all("set-cookie")
        .iter()
        .filter_map(|c| c.to_str().ok())
        .map(|c| {
            let parts: Vec<_> = c.split(';').next().unwrap().split('=').collect();
            (parts[0], parts[1])
        })
        .collect();

    // Test logout endpoint
    let (status, _, headers) = test_request(
        app.clone(),
        "POST",
        "/logout",
        None,
        None,
        Some(&cookies),
    ).await;

    assert_eq!(status, StatusCode::OK);

    // Verify that the cookies are cleared
    let clear_cookies: Vec<_> = headers.get_all("set-cookie")
        .iter()
        .filter_map(|c| c.to_str().ok())
        .collect();
    assert_eq!(clear_cookies.len(), 2);
    assert!(clear_cookies.iter().all(|c| c.contains("expires")));

    // Try to use the cookies after logout
    let (status, _, _) = test_request(
        app,
        "POST",
        "/refresh",
        None,
        None,
        Some(&cookies),
    ).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
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
            let parts: Vec<_> = c.split(';').next().unwrap().split('=').collect();
            (parts[0], parts[1])
        })
        .collect();

    // Test /me endpoint with cookies
    let (status, body, _) = test_request(
        app,
        "GET",
        "/me",
        None,
        None,
        Some(&cookies),
    ).await;

    let user_response: Value = serde_json::from_str(&body).unwrap();

    assert_eq!(status, StatusCode::OK);
    assert_eq!(user_response["username"], username);
    assert_eq!(user_response["email"], email);
    assert!(user_response["id"].is_number());
} 