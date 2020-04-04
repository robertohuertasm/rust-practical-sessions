use crate::proto::User;
use anyhow::Result;
use prost_types::{Timestamp};

use sqlx::{
    types::chrono::{DateTime, NaiveDate, Utc},
    PgPool,
};
use std::collections::HashMap;

#[tonic::async_trait]
pub trait Repository {
    async fn get_user(&self, name: &str) -> Result<User>;
}

pub struct PostgresRepository {
    pub pool: PgPool,
}

impl PostgresRepository {
    pub async fn build(conn_str: &str) -> Result<Self> {
        let pool = PgPool::new(conn_str).await?;
        Ok(Self { pool })
    }
}

#[tonic::async_trait]
#[allow(clippy::empty_line_after_outer_attr)]
impl Repository for PostgresRepository {
    async fn get_user(&self, name: &str) -> Result<User> {
        sqlx::query_as!(
          RawUser, 
          "SELECT id, name, birth_date, created_at, updated_at, custom_data FROM users where name = $1", 
          name
        )
        .fetch_one(&self.pool)
        .await.map(RawUser::into).map_err(sqlx::Error::into)
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
            custom_data: to_map(&raw_user.custom_data),
        }
    }
}

fn to_map(json: &serde_json::Value) -> HashMap<String, i64> {
    let mut map = HashMap::new();
    for (key, val) in json.as_object().unwrap() {
            map.insert(key.to_owned(), val.as_i64().unwrap());
    }
    map
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
