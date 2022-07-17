use chrono::{DateTime, Utc};
use cosmo_store::common::i64_event_version::EventVersion;
use cosmo_store::types::event_stream::EventStream;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DBEventStream {
    pub id: String,
    pub last_version: i64,
    pub last_updated_utc: DateTime<Utc>,
}

impl From<DBEventStream> for EventStream<EventVersion> {
    fn from(s: DBEventStream) -> Self {
        EventStream {
            id: s.id,
            last_version: EventVersion::new(s.last_version),
            last_updated_utc: s.last_updated_utc,
        }
    }
}

impl From<EventStream<EventVersion>> for DBEventStream {
    fn from(s: EventStream<EventVersion>) -> Self {
        DBEventStream {
            id: s.id,
            last_version: s.last_version.0,
            last_updated_utc: s.last_updated_utc,
        }
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DBEventData {
    pub(crate) id: Uuid,
    pub(crate) correlation_id: Option<Uuid>,
    pub(crate) causation_id: Option<Uuid>,
    pub(crate) stream_id: String,
    pub(crate) version: i64,
    pub(crate) name: String,
    pub(crate) data: serde_json::Value,
    pub(crate) metadata: Option<serde_json::Value>,
    pub(crate) created_utc: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DBCommandData {
    pub(crate) id: Uuid,
    pub(crate) correlation_id: Uuid,
    pub(crate) causation_id: Uuid,
    pub(crate) data: serde_json::Value,
    pub(crate) name: String,
    pub(crate) created_utc: DateTime<Utc>,
}
