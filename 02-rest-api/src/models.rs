use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct User {
    pub id: Option<Uuid>,
    pub name: String,
    pub birth_date: NaiveDate,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub custom_data: Option<Json<CustomData>>,
}

impl Default for User {
    fn default() -> Self {
        Self {
            id: None,
            name: String::default(),
            birth_date: NaiveDate::from_ymd(1977, 03, 10),
            created_at: None,
            updated_at: None,
            custom_data: None,
        }
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CustomData {
    pub random: u32,
}
