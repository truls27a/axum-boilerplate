use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use dotenv::dotenv;
use std::env;
use tracing::{info, error};

pub mod redis;
pub use redis::RedisStore;

pub async fn create_db_pool() -> Result<SqlitePool, sqlx::Error> {
    // Load .env file
    dotenv().ok();
    
    // Get database URL from environment variable
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    info!(url = %db_url, "Initializing database connection");
    
    // Create connection pool
    let pool = match SqlitePool::connect(&db_url).await {
        Ok(pool) => {
            info!("Successfully connected to database");
            pool
        }
        Err(e) => {
            error!(error = %e, "Failed to connect to database");
            return Err(e);
        }
    };
    
    // Run migrations
    info!("Running database migrations");
    match sqlx::migrate!().run(&pool).await {
        Ok(_) => {
            info!("Successfully ran database migrations");
            Ok(pool)
        }
        Err(e) => {
            error!(error = %e, "Failed to run database migrations");
            Err(e.into())
        }
    }
}

pub fn create_redis_store() -> Result<RedisStore, ::redis::RedisError> {
    RedisStore::new()
} 