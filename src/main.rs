use axum::{
    routing::{get, post},
    Json, Router,
    extract::State,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use sqlx::SqlitePool;

mod db;
mod api;
mod models;
mod utils;

#[derive(Serialize)]
struct Message {
    message: String,
}

async fn hello_world() -> Json<Message> {
    Json(Message {
        message: "Hello, World!".to_string(),
    })
}


#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Initialize database
    let pool = db::create_db_pool().await;

    // Create a CORS layer
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // build our application with routes
    let app = Router::new()
        .route("/", get(hello_world))
        .route("/login", post(api::auth::login))
        .layer(cors)
        .with_state(pool);

    // run it with hyper
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
