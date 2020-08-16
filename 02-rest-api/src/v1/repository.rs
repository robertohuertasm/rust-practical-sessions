use crate::models::{CustomData, User};
use async_trait::async_trait;
use sqlx::{types::chrono::Utc, types::Json, PgPool, Result};
use std::time::Instant;
use tracing::{self as log, instrument};

#[async_trait]
pub trait Repository: std::fmt::Debug {
    /// Gets a user by id from the database.
    async fn get_user(&self, id: &uuid::Uuid) -> Result<User>;
    /// Creates a new user in the database.
    async fn create_user(&self, user: User) -> Result<User>;
    /// Updates the user's custom_data field.
    async fn update_user(&self, id: &uuid::Uuid, custom_data: CustomData) -> Result<User>;
    /// Deletes a user.
    async fn delete_user(&self, id: &uuid::Uuid) -> Result<User>;
}

/// Postgres repository implementation
#[derive(Debug)]
pub struct PostgresRepository {
    pub pool: PgPool,
}

impl PostgresRepository {
    /// Generates a connection pool for a Postgres database
    pub async fn build(conn_str: &str) -> Result<Self> {
        let pool = PgPool::connect(conn_str).await?;
        Ok(Self { pool })
    }

    /// Generates a connection pool for a Postgres database
    /// by using an env variable for the connection string.
    /// The env var must be called [DATABASE_URL].
    pub async fn build_from_env() -> Result<Self> {
        let conn_str =
            std::env::var("DATABASE_URL").map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;
        PostgresRepository::build(&conn_str).await
    }
}

#[async_trait]
impl Repository for PostgresRepository {
    #[instrument]
    async fn get_user(&self, id: &uuid::Uuid) -> Result<User> {
        measure_query!("Get", {
            sqlx::query_as!(
                User,
                r#"
                SELECT id as "id?", name, birth_date, custom_data as "custom_data: Json<CustomData>", created_at, updated_at
                FROM users
                WHERE id = $1
                "#,
                id,
            )
            .fetch_one(&self.pool)
            .await
        })
    }

    #[instrument]
    async fn create_user(&self, user: User) -> Result<User> {
        measure_query!("Create", {
            sqlx::query_as!(
                User,
                r#"
            INSERT INTO users (name, birth_date, custom_data)
            VALUES ($1, $2, $3)
            RETURNING id as "id?", name, birth_date, custom_data as "custom_data: Json<CustomData>", created_at, updated_at
            "#,
                user.name,
                user.birth_date,
                user.custom_data as _,
            )
            .fetch_one(&self.pool)
            .await
        })
    }

    #[instrument]
    async fn update_user(&self, id: &uuid::Uuid, custom_data: CustomData) -> Result<User> {
        measure_query!("Update", {
            sqlx::query_as!(
                User,
                r#"
            UPDATE users
            SET custom_data = $1, updated_at = $2
            WHERE id = $3
            RETURNING id  as "id?", name, birth_date, custom_data as "custom_data: Json<CustomData>", created_at, updated_at
            "#,
                Json(custom_data) as _,
                Utc::now(),
                id,
            )
            .fetch_one(&self.pool)
            .await
        })
    }

    #[instrument]
    async fn delete_user(&self, id: &uuid::Uuid) -> Result<User> {
        measure_query!("Delete", {
            sqlx::query_as!(
                User,
                r#"
            DELETE FROM users
            WHERE id = $1
            RETURNING id  as "id?", name, birth_date, custom_data as "custom_data: Json<CustomData>", created_at, updated_at
            "#,
                id,
            )
            .fetch_one(&self.pool)
            .await
        })
    }
}
