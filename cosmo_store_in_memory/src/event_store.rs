use anyhow::{bail, Result};
use async_trait::async_trait;
use cosmo_store::common::u32_event_version::{event_writes_to_reads, updated_stream, EventVersion};
use cosmo_store::traits::event_store::EventStore;
use cosmo_store::traits::version::Version;
use cosmo_store::types::event_read::EventRead;
use cosmo_store::types::event_read_range::EventsReadRange;
use cosmo_store::types::event_stream::EventStream;
use cosmo_store::types::event_write::EventWrite;
use cosmo_store::types::expected_version::ExpectedVersion;
use cosmo_store::types::stream_read_filter::StreamsReadFilter;
use std::collections::HashMap;
use uuid::Uuid;

pub struct EventStoreInMemory<Payload, Meta, Version> {
    streams: HashMap<String, EventStream<Version>>,
    events: HashMap<String, EventRead<Payload, Meta, Version>>,
}

impl<Payload: Clone, Meta: Clone> Default for EventStoreInMemory<Payload, Meta, EventVersion> {
    fn default() -> Self {
        EventStoreInMemory::new()
    }
}

impl<Payload: Clone, Meta: Clone> EventStoreInMemory<Payload, Meta, EventVersion> {
    pub fn new() -> EventStoreInMemory<Payload, Meta, EventVersion> {
        EventStoreInMemory {
            streams: HashMap::new(),
            events: HashMap::new(),
        }
    }

    fn get_stream_values(&self, keys: Vec<String>) -> Vec<EventStream<EventVersion>> {
        let res: Vec<EventStream<EventVersion>> = keys
            .iter()
            .map(|x| self.streams.get(x).unwrap())
            .cloned()
            .collect();
        res
    }
    async fn process_events(
        &mut self,
        stream_id: &str,
        version: &ExpectedVersion<EventVersion>,
        payload: Vec<EventWrite<Payload, Meta>>,
    ) -> Result<Vec<EventRead<Payload, Meta, EventVersion>>> {
        let exist: Option<&EventStream<EventVersion>> = self.streams.get(stream_id);
        let last: (EventVersion, Option<EventStream<EventVersion>>) = match exist {
            Some(r) => (r.last_version.clone(), Some(r.clone())),
            None => (EventVersion::new(0), None),
        };

        let next = last.0.next_version(version)?;

        let ops: Vec<EventRead<Payload, Meta, EventVersion>> =
            event_writes_to_reads(stream_id, &next, &payload);
        let updated_stream = updated_stream(stream_id, payload.len() as u32, last);
        // Updating stream
        self.streams
            .entry(stream_id.to_string())
            .or_insert(updated_stream);
        // Updating EVENTS
        ops.iter().for_each(|x| {
            let _ = self.events.insert(x.id.to_string(), x.clone());
        });
        Ok(ops)
    }
}

#[async_trait]
impl<Payload: Clone, Meta: Clone> EventStore<Payload, Meta, EventVersion>
    for EventStoreInMemory<Payload, Meta, EventVersion>
where
    Payload: Send + Sync + 'static,
    Meta: Send + Sync + 'static,
{
    async fn append_event(
        &mut self,
        stream_id: &str,
        version: &ExpectedVersion<EventVersion>,
        payload: &EventWrite<Payload, Meta>,
    ) -> Result<EventRead<Payload, Meta, EventVersion>> {
        let res = self
            .append_events(stream_id, version, vec![payload.clone()])
            .await?;
        Ok(res[0].clone())
    }

    async fn append_events(
        &mut self,
        stream_id: &str,
        version: &ExpectedVersion<EventVersion>,
        payload: Vec<EventWrite<Payload, Meta>>,
    ) -> Result<Vec<EventRead<Payload, Meta, EventVersion>>> {
        if payload.is_empty() {
            return Ok(Vec::new());
        }

        self.process_events(stream_id, version, payload).await
    }

    async fn get_event(
        &self,
        stream_id: &str,
        version: &EventVersion,
    ) -> Result<EventRead<Payload, Meta, EventVersion>> {
        let filter = EventsReadRange::VersionRange {
            from_version: version.clone(),
            to_version: EventVersion(version.0 + 1),
        };
        let events = self.get_events(stream_id, &filter).await?;
        Ok(events[0].clone())
    }

    async fn get_events(
        &self,
        stream_id: &str,
        version: &EventsReadRange<EventVersion>,
    ) -> Result<Vec<EventRead<Payload, Meta, EventVersion>>> {
        let events: Vec<EventRead<Payload, Meta, EventVersion>> = self
            .events
            .values()
            .filter(|x| x.stream_id == *stream_id)
            .cloned()
            .collect();

        let mut fetch = match version {
            EventsReadRange::AllEvents => events,
            EventsReadRange::FromVersion(v) => events
                .iter()
                .filter(|p| p.version.0 >= v.0)
                .cloned()
                .collect(),
            EventsReadRange::ToVersion(v) => events
                .iter()
                .filter(|p| p.version.0 > 0 && p.version.0 <= v.0)
                .cloned()
                .collect(),
            EventsReadRange::VersionRange {
                from_version,
                to_version,
            } => events
                .iter()
                .filter(|p| p.version.0 >= from_version.0 && p.version.0 <= to_version.0)
                .cloned()
                .collect(),
        };

        fetch.sort_by(|a, b| a.version.0.cmp(&b.version.0));
        Ok(fetch)
    }

    async fn get_events_by_correlation_id(
        &self,
        correlation_id: &Uuid,
    ) -> Result<Vec<EventRead<Payload, Meta, EventVersion>>> {
        Ok(self
            .events
            .values()
            .filter(|x| x.correlation_id == Some(*correlation_id))
            .cloned()
            .collect())
    }

    async fn get_events_by_causation_id(
        &self,
        causation_id: &Uuid,
    ) -> Result<Vec<EventRead<Payload, Meta, EventVersion>>> {
        Ok(self
            .events
            .values()
            .filter(|x| x.causation_id == Some(*causation_id))
            .cloned()
            .collect())
    }

    async fn get_streams(
        &self,
        filter: &StreamsReadFilter,
    ) -> Result<Vec<EventStream<EventVersion>>> {
        match filter {
            StreamsReadFilter::AllStreams => Ok(self.streams.values().cloned().collect()),
            StreamsReadFilter::StartsWith(c) => {
                let keys: Vec<String> = self
                    .streams
                    .keys()
                    .filter(|p| p.starts_with(c))
                    .map(|x| x.to_string())
                    .collect();
                let res = self.get_stream_values(keys);
                Ok(res)
            }
            StreamsReadFilter::EndsWith(c) => {
                let keys = self
                    .streams
                    .keys()
                    .filter(|p| p.ends_with(c))
                    .map(|x| x.to_string())
                    .collect();
                let res = self.get_stream_values(keys);
                Ok(res)
            }
            StreamsReadFilter::Contains(c) => {
                let keys = self
                    .streams
                    .keys()
                    .filter(|p| p.contains(c))
                    .map(|x| x.to_string())
                    .collect();
                let res = self.get_stream_values(keys);
                Ok(res)
            }
        }
    }

    async fn get_stream(&self, stream_id: &str) -> Result<EventStream<EventVersion>> {
        let res = self.streams.get(stream_id);
        match res {
            None => {
                bail!("StreamID: {} not present in store", stream_id.to_string())
            }
            Some(r) => Ok(r.clone()),
        }
    }
}
