use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;


pub async fn create_db_pool() -> SqlitePool {
    // Create SQLite database file if it doesn't exist
    let db_url = "sqlite:users.db";
    
    // Create connection pool
    let pool = SqlitePool::connect(db_url).await.unwrap();
    
    // Run migrations
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run database migrations");
        
    pool
}