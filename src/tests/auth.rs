#[cfg(test)]
use axum::{
    http::{Request, StatusCode, HeaderMap},
    body::Body,
};
use serde_json::{json, Value};
use super::helpers::{setup_test_db, create_test_app, test_request};
use tower::ServiceExt;
use crate::services::cookie_service::{CookieService, ACCESS_TOKEN_COOKIE, REFRESH_TOKEN_COOKIE};

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
    ).await;

    assert_eq!(status, StatusCode::OK);
    
    // Parse response and verify message
    let response: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(response["message"], "Login successful");
    
    // Verify both cookies are present
    let cookies: Vec<_> = headers.get_all("set-cookie").iter().collect();
    assert_eq!(cookies.len(), 2);
    
    let cookies_str = cookies.iter()
        .filter_map(|c| c.to_str().ok())
        .collect::<Vec<_>>()
        .join("; ");
    
    assert!(cookies_str.contains(ACCESS_TOKEN_COOKIE));
    assert!(cookies_str.contains(REFRESH_TOKEN_COOKIE));
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

    let (_, _, _) = test_request(
        app.clone(),
        "POST",
        "/register",
        Some(register_data),
        None,
    ).await;

    let login_data = json!({
        "email": email,
        "password": password,
    });

    let (_, _, login_headers) = test_request(
        app.clone(),
        "POST",
        "/login",
        Some(login_data),
        None,
    ).await;

    // Extract refresh token cookie from login response
    let cookies: Vec<_> = login_headers.get_all("set-cookie").iter().collect();
    let cookies_str = cookies.iter()
        .filter_map(|c| c.to_str().ok())
        .collect::<Vec<_>>()
        .join("; ");

    // Create headers for refresh request
    let mut refresh_headers = HeaderMap::new();
    refresh_headers.insert("cookie", cookies_str.parse().unwrap());

    // Test refresh token endpoint
    let (status, body, headers) = test_request(
        app,
        "POST",
        "/refresh",
        None,
        Some(refresh_headers),
    ).await;

    assert_eq!(status, StatusCode::OK);
    let response: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(response["message"], "Tokens refreshed successfully");

    // Verify new cookies are present
    let new_cookies: Vec<_> = headers.get_all("set-cookie").iter().collect();
    assert_eq!(new_cookies.len(), 2);
}

#[tokio::test]
async fn test_refresh_token_invalid() {
    let pool = setup_test_db().await;
    let app = create_test_app(pool);

    // Try to refresh with invalid token
    let mut headers = HeaderMap::new();
    headers.insert("cookie", format!("{}=invalid_token", REFRESH_TOKEN_COOKIE).parse().unwrap());

    let (status, _, _) = test_request(
        app,
        "POST",
        "/refresh",
        None,
        Some(headers),
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
    ).await;

    let login_data = json!({
        "email": email,
        "password": password,
    });

    let (_, _, login_headers) = test_request(
        app.clone(),
        "POST",
        "/login",
        Some(login_data),
        None,
    ).await;

    // Extract cookies from login response
    let cookies: Vec<_> = login_headers.get_all("set-cookie").iter().collect();
    let cookies_str = cookies.iter()
        .filter_map(|c| c.to_str().ok())
        .collect::<Vec<_>>()
        .join("; ");

    // Create headers for logout request
    let mut logout_headers = HeaderMap::new();
    logout_headers.insert("cookie", cookies_str.parse().unwrap());

    // Test logout endpoint
    let (status, _, headers) = test_request(
        app.clone(),
        "POST",
        "/logout",
        None,
        Some(logout_headers),
    ).await;

    assert_eq!(status, StatusCode::OK);

    // Verify that the cookies are cleared
    let clear_cookies: Vec<_> = headers.get_all("set-cookie").iter().collect();
    assert_eq!(clear_cookies.len(), 2);
    
    let clear_cookies_str = clear_cookies.iter()
        .filter_map(|c| c.to_str().ok())
        .collect::<Vec<_>>()
        .join("; ");
    
    // Cookies should be expired
    assert!(clear_cookies_str.contains("expires"));

    // Try to use the refresh token after logout
    let mut refresh_headers = HeaderMap::new();
    refresh_headers.insert("cookie", cookies_str.parse().unwrap());

    let (status, _, _) = test_request(
        app,
        "POST",
        "/refresh",
        None,
        Some(refresh_headers),
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
    ).await;

    // Login to get cookies
    let login_data = json!({
        "email": email,
        "password": password,
    });

    let (_, _, login_headers) = test_request(
        app.clone(),
        "POST",
        "/login",
        Some(login_data),
        None,
    ).await;

    // Extract cookies from login response
    let cookies: Vec<_> = login_headers.get_all("set-cookie").iter().collect();
    let cookies_str = cookies.iter()
        .filter_map(|c| c.to_str().ok())
        .collect::<Vec<_>>()
        .join("; ");

    // Test /me endpoint with cookies
    let mut headers = HeaderMap::new();
    headers.insert("cookie", cookies_str.parse().unwrap());

    let (status, body, _) = test_request(
        app,
        "GET",
        "/me",
        None,
        Some(headers),
    ).await;

    let user_response: Value = serde_json::from_str(&body).unwrap();

    assert_eq!(status, StatusCode::OK);
    assert_eq!(user_response["username"], username);
    assert_eq!(user_response["email"], email);
    assert!(user_response["id"].is_number());
} 