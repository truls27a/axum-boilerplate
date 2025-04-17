use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use dotenv::dotenv;
use std::env;

pub mod redis;
pub use redis::RedisStore;

pub async fn create_db_pool() -> SqlitePool {
    // Load .env file
    dotenv().ok();
    
    // Get database URL from environment variable
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    
    // Create connection pool
    let pool = SqlitePool::connect(&db_url).await.unwrap();
    
    // Run migrations
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run database migrations");
        
    pool
}

pub fn create_redis_store() -> RedisStore {
    RedisStore::new().expect("Failed to create Redis store")
} 