use crate::proto::User;
use anyhow::Result;
use sqlx::PgPool;

pub struct PostgresRepository {
    pub pool: PgPool,
}

impl PostgresRepository {
    pub async fn build(conn_str: &str) -> Result<Self> {
        let pool = PgPool::new(conn_str).await?;
        Ok(Self { pool })
    }

    pub async fn get_user(&self, name: &str) -> Result<User> {
        let pool = &self.pool;

        let raw_user = sqlx::query_as!(RawUser, "SELECT id, name FROM users where name = $1", name)
            .fetch_one(pool)
            .await?;

        println!("{:?}", raw_user);

        Ok(raw_user.into())
    }
}

#[derive(Debug)]
struct RawUser {
    pub id: uuid::Uuid,
    pub name: String,
}

impl From<RawUser> for User {
    fn from(raw_user: RawUser) -> Self {
        Self {
            id: raw_user.id.to_string(),
            name: raw_user.name,
            birth_date: None,
            created_at: None,
            updated_at: None,
        }
    }
}
