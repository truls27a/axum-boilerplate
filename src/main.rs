use axum::{
    routing::{get, post},
    Json, Router,
    extract::State,
    middleware::from_fn_with_state,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use http::{
    Method, 
    HeaderValue, 
    HeaderName,
    header::{AUTHORIZATION, ACCEPT, CONTENT_TYPE},
};
use sqlx::SqlitePool;
use tracing::{info, warn, error, Level};
use std::time::Duration;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod db;
mod api;
mod models;
mod utils;
mod services;
mod middleware;
#[cfg(test)]
mod tests;

use services::jwt_service::JwtService;

#[derive(Clone)]
pub struct AppState {
    db: SqlitePool,
    redis: db::RedisStore,
    jwt_service: JwtService,
}

#[derive(Serialize)]
struct Message {
    message: String,
}

async fn hello_world() -> Json<Message> {
    Json(Message {
        message: "Hello, World!".to_string(),
    })
}

pub fn create_router(pool: SqlitePool, redis_store: db::RedisStore) -> Router {
    // Get the secret key from environment
    dotenv::dotenv().ok();
    let secret_key = std::env::var("SECRET_KEY").expect("SECRET_KEY must be set");
    
    // Create the JWT service
    let jwt_service = JwtService::new(redis_store.clone(), secret_key);

    let state = AppState {
        db: pool,
        redis: redis_store,
        jwt_service,
    };

    // Create a CORS layer
    let cors = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE])
        .allow_credentials(true)
        .max_age(Duration::from_secs(3600));

    // Create protected routes
    let protected_routes = Router::new()
        .route("/me", get(api::user::get_current_user))
        .layer(from_fn_with_state(state.clone(), middleware::auth::auth_middleware));

    // build our application with routes
    Router::new()
        .route("/", get(hello_world))
        .route("/login", post(api::auth::login))
        .route("/register", post(api::auth::register))
        .route("/refresh", post(api::auth::refresh_token))
        .route("/logout", post(api::auth::logout))
        .merge(protected_routes)
        .layer(cors)
        .with_state(state)
}

#[tokio::main]
async fn main() {
    // Initialize structured logging with timestamp and log level
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(true)
        .with_level(true)
        .with_file(true)
        .with_line_number(true)
        .with_thread_names(true)
        .with_max_level(Level::INFO)
        .init();

    info!("Starting Axum API server...");

    // Initialize database
    info!("Initializing database connection...");
    let pool = match db::create_db_pool().await {
        Ok(pool) => {
            info!("Successfully connected to database");
            pool
        },
        Err(e) => {
            error!("Failed to connect to database: {}", e);
            std::process::exit(1);
        }
    };
    
    // Initialize Redis
    info!("Initializing Redis connection...");
    let redis_store = match db::create_redis_store() {
        Ok(store) => {
            info!("Successfully connected to Redis");
            store
        },
        Err(e) => {
            error!("Failed to connect to Redis: {}", e);
            std::process::exit(1);
        }
    };

    // Create the router
    info!("Configuring API routes...");
    let app = create_router(pool, redis_store);

    // run it with hyper
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("🚀 Server starting on http://{}", addr);
    
    match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => {
            info!("Successfully bound to port 3000");
            if let Err(e) = axum::serve(listener, app).await {
                error!("Server error: {}", e);
                std::process::exit(1);
            }
        },
        Err(e) => {
            error!("Failed to bind to port 3000: {}", e);
            std::process::exit(1);
        }
    }
}
