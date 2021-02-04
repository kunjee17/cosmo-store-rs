use chrono::{DateTime, Utc};

pub struct DBEventStream {
    pub id: String,
    pub last_version: u32,
    pub last_updated_utc: DateTime<Utc>
}
