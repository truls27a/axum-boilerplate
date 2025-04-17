use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
    extract::State,
};
use tower::ServiceExt;
use serde_json::Value;
use sqlx::SqlitePool;

pub async fn setup_test_db() -> SqlitePool {
    // Create a new in-memory SQLite database for testing
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("Failed to create test database");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

pub fn create_test_app(pool: SqlitePool) -> Router {
    super::super::create_router(pool)
}

pub async fn test_request(
    app: Router,
    method: &str,
    uri: &str,
    body: Option<Value>,
) -> (StatusCode, String) {
    let body = if let Some(json) = body {
        Body::from(serde_json::to_string(&json).unwrap())
    } else {
        Body::empty()
    };

    let request = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .body(body)
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body = String::from_utf8(bytes.to_vec()).unwrap();

    (status, body)
} 