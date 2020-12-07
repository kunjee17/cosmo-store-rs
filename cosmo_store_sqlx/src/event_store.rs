use crate::event_version::EventVersion;
use anyhow::{bail, Result};
use async_trait::async_trait;
use cosmo_store::{
    EventRead, EventStore, EventStream, EventWrite, EventsReadRange, ExpectedVersion,
    StreamsReadFilter, Version,
};
use sqlx::types::Uuid;
use sqlx::PgPool;

pub struct EventStoreSQLX {
    pool: PgPool,
    event_table: String,
}

#[async_trait]
impl<Payload, Meta> EventStore<Payload, Meta, EventVersion> for EventStoreSQLX
    where
        Payload: Default + Send + Sync + 'static,
        Meta: Default + Send + Sync + 'static,
{
    async fn append_event(
        &self,
        stream_id: &str,
        version: &ExpectedVersion<EventVersion>,
        payload: &EventWrite<Payload, Meta>,
    ) -> Result<EventRead<Payload, Meta, EventVersion>> {
        bail!("unimplemented")
    }

    async fn append_events(
        &self,
        stream_id: &str,
        version: &ExpectedVersion<EventVersion>,
        payload: Vec<EventWrite<Payload, Meta>>,
    ) -> Result<Vec<EventRead<Payload, Meta, EventVersion>>> {
        bail!("unimplemented")
    }

    async fn get_event(
        &self,
        stream_id: &str,
        version: &EventVersion,
    ) -> Result<EventRead<Payload, Meta, EventVersion>> {
        bail!("unimplemented")
    }

    async fn get_events(
        &self,
        stream_id: &str,
        version: &EventsReadRange<EventVersion>,
    ) -> Result<Vec<EventRead<Payload, Meta, EventVersion>>> {
        bail!("unimplemented")
    }

    async fn get_events_by_correlation_id(
        &self,
        correlation_id: &Uuid,
    ) -> Result<Vec<EventRead<Payload, Meta, EventVersion>>> {
        bail!("unimplemented")
    }

    async fn get_streams(
        &self,
        filter: &StreamsReadFilter,
    ) -> Result<Vec<EventStream<EventVersion>>> {
        bail!("unimplemented")
    }

    async fn get_stream(&self, stream_id: &str) -> Result<EventStream<EventVersion>> {
        bail!("unimplemented")
    }
}
