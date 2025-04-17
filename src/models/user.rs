use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use bcrypt::{hash, DEFAULT_COST};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub email: String,
}

impl User {
    pub async fn create(
        pool: &SqlitePool,
        username: &str,
        password: &str,
        email: &str,
    ) -> Result<User, sqlx::Error> {
        let password_hash = hash(password.as_bytes(), DEFAULT_COST)
            .map_err(|_| sqlx::Error::Protocol("Failed to hash password".into()))?;

        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (username, password_hash, email)
            VALUES (?, ?, ?)
            RETURNING 
                id as "id!", 
                username as "username!", 
                password_hash as "password_hash!",
                email as "email!"
            "#,
            username,
            password_hash,
            email
        )
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    pub async fn find_by_email(pool: &SqlitePool, email: &str) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as!(
            User,
            r#"
            SELECT 
                id as "id!", 
                username as "username!", 
                password_hash as "password_hash!",
                email as "email!"
            FROM users
            WHERE email = ?
            "#,
            email
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_id(pool: &SqlitePool, user_id: i64) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as!(
            User,
            r#"
            SELECT 
                id as "id!", 
                username as "username!", 
                password_hash as "password_hash!",
                email as "email!"
            FROM users
            WHERE id = ?
            "#,
            user_id
        )
        .fetch_optional(pool)
        .await
    }    
} 