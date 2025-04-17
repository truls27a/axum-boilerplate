#[cfg(test)]
use axum::{
    http::{Request, StatusCode},
    body::Body,
};
use serde_json::{json, Value};
use super::helpers::{setup_test_db, create_test_app, test_request};
use tower::ServiceExt;

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

    let (status, body) = test_request(
        app,
        "POST",
        "/register",
        Some(register_data),
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

    let (status, _) = test_request(
        app.clone(),
        "POST",
        "/register",
        Some(register_data),
    ).await;

    assert_eq!(status, StatusCode::OK);

    // Now test login
    let login_data = json!({
        "email": email,
        "password": password,
    });

    let (status, body) = test_request(
        app,
        "POST",
        "/login",
        Some(login_data),
    ).await;

    assert_eq!(status, StatusCode::OK);
    
    // Parse response and verify token exists
    let response: Value = serde_json::from_str(&body).unwrap();
    assert!(response.get("token").is_some());
}

#[tokio::test]
async fn test_login_invalid_password() {
    // Setup
    let pool = setup_test_db().await;
    let app = create_test_app(pool.clone());

    let username = "testuser";
    let email = "test@example.com";
    let password = "password123";
    let wrong_password = "wrongpassword";

    // First register a user
    let register_data = json!({
        "username": username,
        "email": email,
        "password": password
    });

    let (status, _) = test_request(
        app.clone(),
        "POST",
        "/register",
        Some(register_data),
    ).await;

    assert_eq!(status, StatusCode::OK);

    // Test login with wrong password
    let login_data = json!({
        "email": email,
        "password": wrong_password,
    });

    let (status, _) = test_request(
        app,
        "POST",
        "/login",
        Some(login_data),
    ).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_login_nonexistent_user() {
    let pool = setup_test_db().await;
    let app = create_test_app(pool);

    let email = "nonexistent@example.com";
    let password = "password123";

    let login_data = json!({
        "email": email,
        "password": password,
    });

    let (status, _) = test_request(
        app,
        "POST",
        "/login",
        Some(login_data),
    ).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_get_current_user() {
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

    let (status, _) = test_request(
        app.clone(),
        "POST",
        "/register",
        Some(register_data),
    ).await;

    assert_eq!(status, StatusCode::OK);

    // Login to get token
    let login_data = json!({
        "email": email,
        "password": password,
    });

    let (status, body) = test_request(
        app.clone(),
        "POST",
        "/login",
        Some(login_data),
    ).await;

    assert_eq!(status, StatusCode::OK);
    let login_response: Value = serde_json::from_str(&body).unwrap();
    let token = login_response["token"].as_str().unwrap();

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