use crate::db_types::{DBEventData, DBEventStream};
use crate::event_store_sqlx_postgres::EventStoreSQLXPostgres;
use anyhow::Result;
use async_trait::async_trait;
use cosmo_store::common::i64_event_version::{event_writes_to_reads, updated_stream, EventVersion};
use cosmo_store::traits::event_store::EventStore;
use cosmo_store::traits::version::Version;
use cosmo_store::types::event_read::EventRead;
use cosmo_store::types::event_read_range::EventsReadRange;
use cosmo_store::types::event_stream::EventStream;
use cosmo_store::types::event_write::EventWrite;
use cosmo_store::types::expected_version::ExpectedVersion;
use cosmo_store::types::stream_read_filter::StreamsReadFilter;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::types::Uuid;

impl EventStoreSQLXPostgres {
    fn db_events_to_event_reads<Payload, Meta>(
        events: &[DBEventData],
    ) -> Result<Vec<EventRead<Payload, Meta, EventVersion>>>
    where
        Payload: Send + Sync + 'static + Clone + Serialize + for<'de> Deserialize<'de>,
        Meta: Send + Sync + 'static + Clone + Serialize + for<'de> Deserialize<'de>,
    {
        let mut event_reads: Vec<EventRead<Payload, Meta, EventVersion>> = Vec::new();
        for d in events {
            let metadata = match d.metadata.clone() {
                None => None,
                Some(v) => {
                    let r = serde_json::from_value(v)?;
                    Some(r)
                }
            };
            let event_read = EventRead {
                id: d.id,
                correlation_id: d.correlation_id,
                causation_id: d.causation_id,
                stream_id: d.stream_id.clone(),
                version: EventVersion::new(d.version),
                name: d.name.clone(),
                data: serde_json::from_value(d.data.clone())?,
                metadata,
                created_utc: d.created_utc,
            };
            event_reads.push(event_read)
        }

        Ok(event_reads)
    }

    async fn process_events<Payload: Clone + Serialize, Meta: Clone + Serialize>(
        &self,
        stream_id: &str,
        version: &ExpectedVersion<EventVersion>,
        payload: Vec<EventWrite<Payload, Meta>>,
    ) -> Result<Vec<EventRead<Payload, Meta, EventVersion>>> {
        let pool = self.pool();
        let exist_query = format!(
            "select * from {0} where id = $1 limit 1",
            self.streams_table_name()
        );
        let exist = sqlx::query_as::<_, DBEventStream>(&exist_query)
            .bind(stream_id)
            .fetch_one(&pool)
            .await;
        let last: (EventVersion, Option<EventStream<EventVersion>>) = match &exist {
            Ok(r) => (
                EventVersion::new(r.last_version),
                Some(EventStream::from(r.clone())),
            ),
            Err(_) => (EventVersion::new(0), None),
        };

        let next = last.0.next_version(version)?;

        let ops: Vec<EventRead<Payload, Meta, EventVersion>> =
            event_writes_to_reads(stream_id, &next, &payload.clone());

        let updated_stream = updated_stream(stream_id, payload.len() as i64, last);

        // Updating all in single transection
        let mut tr = pool.begin().await?;

        let insert_or_update_stream = format!("insert into {0} (id, last_version) values ($1, $2) on conflict (id) do update set last_version = $2", self.streams_table_name());
        let _ = sqlx::query(&insert_or_update_stream)
            .bind(updated_stream.id)
            .bind(updated_stream.last_version.0)
            .execute(&mut tr)
            .await?;

        let insert_event = format!("insert into {0} (id, correlation_id, causation_id, stream_id, version, name, data, metadata) values ($1, $2, $3, $4, $5, $6, $7, $8)", self.events_table_name());

        for op in &ops {
            let data = serde_json::to_value(op.data.clone())?;
            let metadata: Option<Value> = match op.metadata.clone() {
                None => None,
                Some(v) => {
                    let r = serde_json::to_value(v)?;
                    Some(r)
                }
            };
            let _ = sqlx::query(&insert_event)
                .bind(op.id)
                .bind(op.correlation_id)
                .bind(op.causation_id)
                .bind(op.stream_id.clone())
                .bind(op.version.0)
                .bind(op.name.clone())
                .bind(data)
                .bind(metadata)
                .execute(&mut tr)
                .await?;
        }

        tr.commit().await?;

        Ok(ops)
    }
}

#[async_trait]
impl<Payload, Meta> EventStore<Payload, Meta, EventVersion> for EventStoreSQLXPostgres
where
    Payload: Send + Sync + 'static + Clone + Serialize + for<'de> Deserialize<'de>,
    Meta: Send + Sync + 'static + Clone + Serialize + for<'de> Deserialize<'de>,
{
    async fn append_event(
        &self,
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
        &self,
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
        match version {
            EventsReadRange::AllEvents => {
                let all_event = format!(
                    "select * from {0} where stream_id=$1",
                    self.events_table_name()
                );
                let db_event_data = sqlx::query_as::<_, DBEventData>(&all_event)
                    .bind(stream_id)
                    .fetch_all(&self.pool())
                    .await?;
                EventStoreSQLXPostgres::db_events_to_event_reads(&db_event_data)
            }
            EventsReadRange::FromVersion(f) => {
                let from_version = format!(
                    "select * from {0} where stream_id=$1 and version >= $2",
                    self.events_table_name()
                );
                let db_event_data = sqlx::query_as::<_, DBEventData>(&from_version)
                    .bind(stream_id)
                    .bind(f.0)
                    .fetch_all(&self.pool())
                    .await?;
                EventStoreSQLXPostgres::db_events_to_event_reads(&db_event_data)
            }
            EventsReadRange::ToVersion(t) => {
                let to_version = format!(
                    "select * from {0} where stream_id=$1 and version <= $2 and version > 0",
                    self.events_table_name()
                );
                let db_event_data = sqlx::query_as::<_, DBEventData>(&to_version)
                    .bind(stream_id)
                    .bind(t.0)
                    .fetch_all(&self.pool())
                    .await?;
                EventStoreSQLXPostgres::db_events_to_event_reads(&db_event_data)
            }
            EventsReadRange::VersionRange {
                from_version,
                to_version,
            } => {
                let version_range = format!(
                    "select * from {0} where stream_id=$1 and version >= $2 and version <= $3",
                    self.events_table_name()
                );
                let db_event_data = sqlx::query_as::<_, DBEventData>(&version_range)
                    .bind(stream_id)
                    .bind(from_version.0)
                    .bind(to_version.0)
                    .fetch_all(&self.pool())
                    .await?;
                EventStoreSQLXPostgres::db_events_to_event_reads(&db_event_data)
            }
        }
    }

    async fn get_events_by_correlation_id(
        &self,
        correlation_id: &Uuid,
    ) -> Result<Vec<EventRead<Payload, Meta, EventVersion>>> {
        let correlation_query = format!(
            "select * from {0} where correlation_id=$1",
            self.events_table_name()
        );
        let db_event_data = sqlx::query_as::<_, DBEventData>(&correlation_query)
            .bind(correlation_id)
            .fetch_all(&self.pool())
            .await?;
        EventStoreSQLXPostgres::db_events_to_event_reads(&db_event_data)
    }

    async fn get_events_by_causation_id(
        &self,
        causation_id: &Uuid,
    ) -> Result<Vec<EventRead<Payload, Meta, EventVersion>>> {
        let correlation_query = format!(
            "select * from {0} where causation_id=$1",
            self.events_table_name()
        );
        let db_event_data = sqlx::query_as::<_, DBEventData>(&correlation_query)
            .bind(causation_id)
            .fetch_all(&self.pool())
            .await?;
        EventStoreSQLXPostgres::db_events_to_event_reads(&db_event_data)
    }

    async fn get_streams(
        &self,
        filter: &StreamsReadFilter,
    ) -> Result<Vec<EventStream<EventVersion>>> {
        match filter {
            StreamsReadFilter::AllStreams => {
                let all_stream = format!("select * from {0}", self.streams_table_name());
                let stream_data = sqlx::query_as::<_, DBEventStream>(&all_stream)
                    .fetch_all(&self.pool())
                    .await?;
                let res: Vec<EventStream<EventVersion>> = stream_data
                    .iter()
                    .map(|x| EventStream::from(x.clone()))
                    .collect();
                Ok(res)
            }
            StreamsReadFilter::StartsWith(s) => {
                let starts_with_stream = format!(
                    "select * from {0} where id like $1%",
                    self.streams_table_name()
                );
                let stream_data = sqlx::query_as::<_, DBEventStream>(&starts_with_stream)
                    .bind(s)
                    .fetch_all(&self.pool())
                    .await?;
                let res: Vec<EventStream<EventVersion>> = stream_data
                    .iter()
                    .map(|x| EventStream::from(x.clone()))
                    .collect();
                Ok(res)
            }
            StreamsReadFilter::EndsWith(s) => {
                let ends_with_stream = format!(
                    "select * from {0} where id like %$1",
                    self.streams_table_name()
                );
                let stream_data = sqlx::query_as::<_, DBEventStream>(&ends_with_stream)
                    .bind(s)
                    .fetch_all(&self.pool())
                    .await?;
                let res: Vec<EventStream<EventVersion>> = stream_data
                    .iter()
                    .map(|x| EventStream::from(x.clone()))
                    .collect();
                Ok(res)
            }
            StreamsReadFilter::Contains(s) => {
                let contains_stream = format!(
                    "select * from {0} where id like %$1%",
                    self.streams_table_name()
                );
                let stream_data = sqlx::query_as::<_, DBEventStream>(&contains_stream)
                    .bind(s)
                    .fetch_all(&self.pool())
                    .await?;
                let res: Vec<EventStream<EventVersion>> = stream_data
                    .iter()
                    .map(|x| EventStream::from(x.clone()))
                    .collect();
                Ok(res)
            }
        }
    }

    async fn get_stream(&self, stream_id: &str) -> Result<EventStream<EventVersion>> {
        let stream_by_id = format!("select * from {0} where id=$1", self.streams_table_name());
        let stream_data = sqlx::query_as::<_, DBEventStream>(&stream_by_id)
            .bind(stream_id)
            .fetch_one(&self.pool())
            .await;
        stream_data.map(EventStream::from).map_err(|_| {
            anyhow::Error::msg(format!(
                "StreamID: {} not present in store",
                stream_id.to_string()
            ))
        })
    }
}
