use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
}

impl User {
    pub async fn find_by_username(pool: &SqlitePool, username: &str) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as!(
            User,
            r#"
            SELECT 
                id as "id!", 
                username as "username!", 
                password_hash as "password_hash!"
            FROM users
            WHERE username = ?
            "#,
            username
        )
        .fetch_optional(pool)
        .await
    }
} 