use anyhow::Result;
use async_trait::async_trait;
use chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub enum ExpectedVersion<Version> {
    Any,
    NoStream,
    Exact(Version),
}

#[derive(Clone, Debug)]
pub enum EventsReadRange<Version> {
    AllEvents,
    FromVersion(Version),
    ToVersion(Version),
    VersionRange {
        from_version: Version,
        to_version: Version,
    },
}

#[derive(Clone, Debug)]
pub enum StreamsReadFilter {
    AllStreams,
    StartsWith(String),
    EndsWith(String),
    Contains(String),
}

#[derive(Clone, Debug)]
pub struct EventStream<Version> {
    pub id: String,
    pub last_version: Version,
    pub last_updated_utc: NaiveDateTime,
    pub created_by: String,
}

#[derive(Clone, Debug)]
pub struct EventWrite<Payload, Meta> {
    pub id: Uuid,
    pub correlation_id: String,
    pub causation_id: String,
    pub name: String,
    pub data: Payload,
    pub metadata: Option<Meta>,
}

#[derive(Clone, Debug)]
pub struct EventRead<Payload, Meta, Version> {
    pub id: Uuid,
    pub correlation_id: String,
    pub causation_id: String,
    pub stream_id: String,
    pub version: Version,
    pub name: String,
    pub data: Payload,
    pub metadata: Option<Meta>,
    pub created_utc: NaiveDateTime,
}

impl<Payload: Copy + Clone, Meta: Copy + Clone, Version> EventRead<Payload, Meta, Version> {
    pub fn from_event_write(
        stream_id: &str,
        version: Version,
        created_utc: NaiveDateTime,
        event_write: &EventWrite<Payload, Meta>,
    ) -> EventRead<Payload, Meta, Version> {
        EventRead {
            id: event_write.id,
            name: event_write.name.to_string(),
            correlation_id: event_write.correlation_id.to_string(),
            causation_id: event_write.causation_id.to_string(),
            stream_id: stream_id.to_string(),
            data: event_write.data,
            metadata: event_write.metadata,
            created_utc,
            version,
        }
    }
}

#[async_trait]
pub trait EventStore<Payload, Meta, Version> {
    async fn append_event(
        &self,
        stream_id: &str,
        version: &ExpectedVersion<Version>,
        payload: &EventWrite<Payload, Meta>,
    ) -> Result<EventRead<Payload, Meta, Version>>;
    async fn append_events(
        &self,
        stream_id: &str,
        version: &ExpectedVersion<Version>,
        payload: Vec<EventWrite<Payload, Meta>>,
    ) -> Result<Vec<EventRead<Payload, Meta, Version>>>;
    async fn get_event(
        &self,
        stream_id: &str,
        version: &Version,
    ) -> Result<EventRead<Payload, Meta, Version>>;
    async fn get_events(
        &self,
        stream_id: &str,
        version: &EventsReadRange<Version>,
    ) -> Result<Vec<EventRead<Payload, Meta, Version>>>;
    async fn get_events_by_correlation_id(
        &self,
        correlation_id: &Uuid,
    ) -> Result<Vec<EventRead<Payload, Meta, Version>>>;
    async fn get_streams(&self, filter: &StreamsReadFilter) -> Result<Vec<EventStream<Version>>>;
    async fn get_stream(&self, stream_id: &str) -> Result<EventStream<Version>>;
}

pub trait Version<Version> {
    // fn validate_version(&self, version: &ExpectedVersion<Version>, next_ver: Version) -> Result<Version>;
    fn next_version(&self, version: &ExpectedVersion<Version>) -> Result<Version>;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
