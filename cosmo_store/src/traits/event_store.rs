use crate::types::event_read::EventRead;
use crate::types::event_read_range::EventsReadRange;
use crate::types::event_stream::EventStream;
use crate::types::event_write::EventWrite;
use crate::types::expected_version::ExpectedVersion;
use crate::types::stream_read_filter::StreamsReadFilter;
use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait EventStore<Payload, Meta, Version>
where
    Version: Eq + PartialEq,
{
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
        range: &EventsReadRange<Version>,
    ) -> Result<Vec<EventRead<Payload, Meta, Version>>>;
    async fn get_events_by_correlation_id(
        &self,
        correlation_id: &Uuid,
    ) -> Result<Vec<EventRead<Payload, Meta, Version>>>;
    async fn get_events_by_causation_id(
        &self,
        causation_id: &Uuid,
    ) -> Result<Vec<EventRead<Payload, Meta, Version>>>;
    async fn get_streams(&self, filter: &StreamsReadFilter) -> Result<Vec<EventStream<Version>>>;
    async fn get_stream(&self, stream_id: &str) -> Result<EventStream<Version>>;
}
