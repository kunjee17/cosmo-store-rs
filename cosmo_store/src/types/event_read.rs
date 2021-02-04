use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::types::event_write::EventWrite;

#[derive(Clone, Debug)]
pub struct EventRead<Payload, Meta, Version> {
    pub id: Uuid,
    pub correlation_id: Option<String>,
    pub causation_id: Option<String>,
    pub stream_id: String,
    pub version: Version,
    pub name: String,
    pub data: Payload,
    pub metadata: Option<Meta>,
    pub created_utc: DateTime<Utc>,
}

impl<Payload: Clone, Meta: Clone, Version> EventRead<Payload, Meta, Version> {
    pub fn from_event_write(
        stream_id: &str,
        version: Version,
        created_utc: DateTime<Utc>,
        event_write: &EventWrite<Payload, Meta>,
    ) -> EventRead<Payload, Meta, Version> {
        EventRead {
            id: event_write.id,
            name: event_write.name.to_string(),
            correlation_id: event_write.correlation_id.clone(),
            causation_id: event_write.causation_id.clone(),
            stream_id: stream_id.to_string(),
            data: event_write.data.clone(),
            metadata: event_write.metadata.clone(),
            created_utc,
            version,
        }
    }
}
