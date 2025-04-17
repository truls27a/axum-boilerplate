use axum::http::StatusCode;
use serde_json::{json, Value};
use crate::models::user::User;
use super::helpers::{setup_test_db, create_test_app, test_request};

async fn create_test_user(pool: &sqlx::SqlitePool, username: &str, password: &str, email: &str) -> User {
    // Create a test user
    User::create(
        pool,
        username,
        password,
        email,
    )
    .await
    .expect("Failed to create test user")
}

#[tokio::test]
async fn test_login_success() {
    // Setup
    let pool = setup_test_db().await;
    let username = "testuser";
    let email = "test@example.com";
    let password = "password123";
    
    // Create user in the test database
    let _user = create_test_user(&pool, username, password, email).await;
    
    // Create app with the same database pool
    let app = create_test_app(pool);

    // Test login
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
    let username = "testuser";
    let email = "test@example.com";
    let password = "password123";
    
    // Create user in the test database
    let _user = create_test_user(&pool, username, password, email).await;
    
    // Create app with the same database pool
    let app = create_test_app(pool);

    // Test login with wrong password
    let login_data = json!({
        "email": email,
        "password": "wrongpassword",
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

    let login_data = json!({
        "email": "nonexistent@example.com",
        "password": "password123",
    });

    let (status, _) = test_request(
        app,
        "POST",
        "/login",
        Some(login_data),
    ).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
} 