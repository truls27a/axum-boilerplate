use crate::db::RedisStore;
use axum::{
    Router,
    body::Body,
    extract::State,
    http::{HeaderMap, Request, StatusCode},
};
use serde_json::Value;
use sqlx::SqlitePool;
use std::sync::Once;
use tower::ServiceExt;
use tower_cookies::Cookie;
use tracing::{Level, info, debug};
use tracing_subscriber::fmt::format::FmtSpan;

static INIT: Once = Once::new();

/// Initialize logging exactly once
pub fn init_tracing() {
    INIT.call_once(|| {
        tracing_subscriber::fmt()
            .with_test_writer()
            .with_target(false)
            .with_thread_ids(true)
            .with_level(true)
            .with_file(true)
            .with_line_number(true)
            .with_thread_names(true)
            .with_max_level(Level::DEBUG)
            .with_span_events(FmtSpan::NONE)
            .init();
    });
}

pub async fn setup_test_db() -> SqlitePool {
    init_tracing();
    info!("Setting up test database");

    // Create a new in-memory SQLite database for testing
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("Failed to create test database");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    info!("Test database setup complete");
    pool
}

pub fn setup_test_redis() -> RedisStore {
    info!("Setting up test Redis store");
    let store = RedisStore::new().expect("Failed to create test Redis store");
    info!("Test Redis setup complete");
    store
}

pub fn create_test_app(pool: SqlitePool) -> Router {
    info!("Creating test application");
    let redis_store = setup_test_redis();
    let app = super::super::create_router(pool, redis_store);
    info!("Test application created");
    app
}

pub async fn test_request(
    app: Router,
    method: &str,
    uri: &str,
    body: Option<Value>,
    headers: Option<HeaderMap>,
    cookies: Option<&[(&str, &str)]>,
) -> (StatusCode, String, HeaderMap) {
    info!(method = %method, uri = %uri, "Making test request");

    let body = if let Some(json) = body {
        info!(body = %json, "Request body");
        Body::from(serde_json::to_string(&json).unwrap())
    } else {
        Body::empty()
    };

    let mut request = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json");

    // Add cookies if provided
    if let Some(cookies) = cookies {
        if !cookies.is_empty() {
            let cookie_header = cookies
                .iter()
                .map(|(name, value)| format!("{}={}", name, value))
                .collect::<Vec<_>>()
                .join("; ");
            debug!("Setting cookie header: {}", cookie_header);
            request = request.header("cookie", cookie_header);
        }
    }

    // Add custom headers if provided
    if let Some(custom_headers) = headers {
        for (key, value) in custom_headers.iter() {
            request = request.header(key, value);
        }
    }

    let request = request.body(body).unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let body = String::from_utf8(
        axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();

    info!(status = %status, body = %body, "Test response received");
    (status, body, headers)
}

/// Helper function to extract cookie values from Set-Cookie headers in test responses
pub fn extract_response_cookie(headers: &HeaderMap, cookie_name: &str) -> Option<String> {
    headers
        .get_all("set-cookie")
        .iter()
        .filter_map(|c| c.to_str().ok())
        .find_map(|c| {
            Cookie::parse(c).ok().and_then(|cookie| {
                if cookie.name() == cookie_name {
                    Some(cookie.value().to_string())
                } else {
                    None
                }
            })
        })
}
