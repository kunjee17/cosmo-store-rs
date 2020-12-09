use crate::event_version::EventVersion;
use anyhow::{bail, Result};
use async_trait::async_trait;
use chrono::Utc;
use cosmo_store::{
    EventRead, EventStore, EventStream, EventWrite, EventsReadRange, ExpectedVersion,
    StreamsReadFilter, Version,
};
use std::collections::HashMap;
use uuid::Uuid;

pub struct EventStoreInMemory<Payload, Meta, Version> {
    streams: HashMap<String, EventStream<Version>>,
    events: HashMap<String, EventRead<Payload, Meta, Version>>,
}

impl<Payload: Clone, Meta: Clone> EventStoreInMemory<Payload, Meta, EventVersion> {
    fn get_stream_values(&self, keys: Vec<String>) -> Vec<EventStream<EventVersion>> {
        let res: Vec<EventStream<EventVersion>> = keys
            .iter()
            .map(|x| self.streams.get(x).unwrap())
            .map(|x| x.clone())
            .collect();
        res
    }
    async fn process_events(
        &mut self,
        stream_id: &str,
        version: &ExpectedVersion<EventVersion>,
        payload: &Vec<EventWrite<Payload, Meta>>,
    ) -> Result<Vec<EventRead<Payload, Meta, EventVersion>>> {
        let exist: Option<&EventStream<EventVersion>> = self.streams.get(stream_id);
        let last: (EventVersion, Option<&EventStream<EventVersion>>) = match exist {
            Some(r) => (r.last_version.clone(), Some(r)),
            None => (EventVersion::new(0), None),
        };

        let next = last.0.next_version(version)?;

        let ops: Vec<EventRead<Payload, Meta, EventVersion>> = payload
            .iter()
            .enumerate()
            .map(|(i, e)| EventRead::from_event_write(stream_id, next.add(i as u32), Utc::now(), e))
            .collect();
        let updated_stream = match last.1 {
            Some(r) => EventStream {
                last_version: r.last_version.add(payload.len() as u32),
                last_updated_utc: Utc::now(),
                ..(r.clone())
            },
            None => EventStream {
                id: stream_id.to_string(),
                last_version: EventVersion::new(payload.len() as u32),
                last_updated_utc: Utc::now(),
            },
        };
        // Updating stream
        self.streams
            .entry(stream_id.to_string())
            .or_insert(updated_stream);
        // Updating events
        ops.iter().for_each(|x| {
            let _ = self.events.insert(x.id.to_string(), x.clone());
            ()
        });
        Ok(ops)
    }
}

#[async_trait]
impl<Payload: Clone, Meta: Clone> EventStore<Payload, Meta, EventVersion>
    for EventStoreInMemory<Payload, Meta, EventVersion>
where
    Payload: Default + Send + Sync + 'static,
    Meta: Default + Send + Sync + 'static,
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
        if payload.len() == 0 {
            return Ok(Vec::new());
        }

        self.process_events(stream_id, version, &payload).await
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
            .filter(|x| x.stream_id == stream_id.to_string())
            .map(|x| x.clone())
            .collect();

        let mut fetch = match version {
            EventsReadRange::AllEvents => events,
            EventsReadRange::FromVersion(v) => events
                .iter()
                .filter(|p| p.version.0 >= v.0)
                .map(|x| x.clone())
                .collect(),
            EventsReadRange::ToVersion(v) => events
                .iter()
                .filter(|p| p.version.0 > 0 && p.version.0 <= v.0)
                .map(|x| x.clone())
                .collect(),
            EventsReadRange::VersionRange {
                from_version,
                to_version,
            } => events
                .iter()
                .filter(|p| p.version.0 >= from_version.0 && p.version.0 <= to_version.0)
                .map(|x| x.clone())
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
            .filter(|x| x.correlation_id == correlation_id.to_string())
            .map(|x| x.clone())
            .collect())
    }

    async fn get_streams(
        &self,
        filter: &StreamsReadFilter,
    ) -> Result<Vec<EventStream<EventVersion>>> {
        match filter {
            StreamsReadFilter::AllStreams => Ok(self.streams.values().map(|p| p.clone()).collect()),
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
