use chrono::{DateTime, Utc};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventStream<Version: Eq + PartialEq> {
    pub id: String,
    pub last_version: Version,
    pub last_updated_utc: DateTime<Utc>,
}
