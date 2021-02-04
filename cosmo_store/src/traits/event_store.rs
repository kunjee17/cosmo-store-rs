use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;
use crate::types::expected_version::ExpectedVersion;
use crate::types::event_write::EventWrite;
use crate::types::event_read::EventRead;
use crate::types::event_read_range::EventsReadRange;
use crate::types::stream_read_filter::StreamsReadFilter;
use crate::types::event_stream::EventStream;

#[async_trait]
pub trait EventStore<Payload, Meta, Version> {
    async fn append_event(
        &mut self,
        stream_id: &str,
        version: &ExpectedVersion<Version>,
        payload: &EventWrite<Payload, Meta>,
    ) -> Result<EventRead<Payload, Meta, Version>>;
    async fn append_events(
        &mut self,
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
