use chrono::{DateTime, Utc};

#[derive(Clone, Debug)]
pub struct EventStream<Version> {
    pub id: String,
    pub last_version: Version,
    pub last_updated_utc: DateTime<Utc>,
}
