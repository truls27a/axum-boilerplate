use axum::{
    routing::{get, post},
    Json, Router,
    extract::State,
    middleware::from_fn_with_state,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use sqlx::SqlitePool;

mod db;
mod api;
mod models;
mod utils;
mod services;
mod middleware;
#[cfg(test)]
mod tests;

#[derive(Serialize)]
struct Message {
    message: String,
}

async fn hello_world() -> Json<Message> {
    Json(Message {
        message: "Hello, World!".to_string(),
    })
}

pub fn create_router(pool: SqlitePool) -> Router {
    // Create a CORS layer
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Create protected routes
    let protected_routes = Router::new()
        .route("/me", get(api::user::get_current_user))
        .layer(from_fn_with_state(pool.clone(), middleware::auth::auth_middleware));

    // build our application with routes
    Router::new()
        .route("/", get(hello_world))
        .route("/login", post(api::auth::login))
        .route("/register", post(api::auth::register))
        .merge(protected_routes)
        .layer(cors)
        .with_state(pool)
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Initialize database
    let pool = db::create_db_pool().await;

    // Create the router
    let app = create_router(pool);

    // run it with hyper
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
