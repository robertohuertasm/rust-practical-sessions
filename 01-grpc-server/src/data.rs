use crate::proto::User;
use anyhow::Result;
use prost_types::Timestamp;

use sqlx::{
    types::chrono::{DateTime, NaiveDate, Utc},
    PgPool,
};

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

        let raw_user = sqlx::query_as!(
            RawUser,
            "SELECT id, name, birth_date, created_at, updated_at, custom_data FROM users where name = $1",
            name
        )
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
    pub birth_date: Option<NaiveDate>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub custom_data: serde_json::Value,
}

impl From<RawUser> for User {
    fn from(raw_user: RawUser) -> Self {
        Self {
            id: raw_user.id.to_string(),
            name: raw_user.name,
            birth_date: naive_to_timestamp(raw_user.birth_date),
            created_at: to_timestamp(raw_user.created_at),
            updated_at: to_timestamp(raw_user.updated_at),
        }
    }
}

fn to_timestamp(datetime: Option<DateTime<Utc>>) -> Option<Timestamp> {
    datetime.map(|dt| Timestamp {
        seconds: dt.timestamp(),
        nanos: 0,
    })
}

fn naive_to_timestamp(date: Option<NaiveDate>) -> Option<Timestamp> {
    date.map(|d| Timestamp {
        seconds: d.and_hms(0, 0, 0).timestamp(),
        nanos: 0,
    })
}
