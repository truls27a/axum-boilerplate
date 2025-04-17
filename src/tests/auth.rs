#[cfg(test)]
use axum::{
    http::{Request, StatusCode, HeaderMap},
    body::Body,
};
use serde_json::{json, Value};
use super::helpers::{setup_test_db, create_test_app, test_request};
use tower::ServiceExt;
use crate::utils::cookies::REFRESH_TOKEN_COOKIE;
use crate::utils::cookies::CookieManager;

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
    
    // Parse response and verify tokens
    let response: Value = serde_json::from_str(&body).unwrap();
    assert!(response.get("access_token").is_some());
    
    // Verify refresh token cookie
    let refresh_token = CookieManager::extract_refresh_token(&headers);
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

    // Extract refresh token from login response
    let refresh_token_cookie = login_headers.get("set-cookie")
        .and_then(|v| v.to_str().ok())
        .expect("Should have refresh token cookie");

    // Create headers for refresh request
    let mut refresh_headers = HeaderMap::new();
    refresh_headers.insert("cookie", refresh_token_cookie.parse().unwrap());

    // Test refresh token endpoint
    let (status, body, _) = test_request(
        app,
        "POST",
        "/refresh",
        None,
        Some(refresh_headers),
    ).await;

    assert_eq!(status, StatusCode::OK);
    let response: Value = serde_json::from_str(&body).unwrap();
    assert!(response.get("access_token").is_some());
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

    // Extract refresh token from login response
    let refresh_token_cookie = login_headers.get("set-cookie")
        .and_then(|v| v.to_str().ok())
        .expect("Should have refresh token cookie");

    // Create headers for logout request
    let mut logout_headers = HeaderMap::new();
    logout_headers.insert("cookie", refresh_token_cookie.parse().unwrap());

    // Test logout endpoint
    let (status, _, headers) = test_request(
        app.clone(),
        "POST",
        "/logout",
        None,
        Some(logout_headers),
    ).await;

    assert_eq!(status, StatusCode::OK);

    // Verify that the refresh token cookie is cleared
    let cookie = headers.get("set-cookie")
        .and_then(|v| v.to_str().ok())
        .expect("Should have cookie header");
    assert!(cookie.contains("Max-Age=0"));

    // Try to use the refresh token after logout
    let mut refresh_headers = HeaderMap::new();
    refresh_headers.insert("cookie", refresh_token_cookie.parse().unwrap());

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

    // Login to get token
    let login_data = json!({
        "email": email,
        "password": password,
    });

    let (_, body, _) = test_request(
        app.clone(),
        "POST",
        "/login",
        Some(login_data),
        None,
    ).await;

    let login_response: Value = serde_json::from_str(&body).unwrap();
    let token = login_response["access_token"].as_str().unwrap();

    // Test /me endpoint with token
    let request = Request::builder()
        .method("GET")
        .uri("/me")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body = String::from_utf8(bytes.to_vec()).unwrap();
    let user_response: Value = serde_json::from_str(&body).unwrap();

    assert_eq!(status, StatusCode::OK);
    assert_eq!(user_response["username"], username);
    assert_eq!(user_response["email"], email);
    assert!(user_response["id"].is_number());
} 