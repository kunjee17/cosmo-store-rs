use crate::event_version::EventVersion;
use anyhow::{bail, Result};
use async_trait::async_trait;
use cosmo_store::{
    EventRead, EventStore, EventStream, EventWrite, EventsReadRange, ExpectedVersion,
    StreamsReadFilter,
};
use sqlx::types::Uuid;
use sqlx::{PgPool, Error};
use sqlx::postgres::PgDone;
use sqlx::types::chrono::Utc;

// Event Read
// pub id: Uuid,
// pub correlation_id: Option<String>,
// pub causation_id: Option<String>,
// pub stream_id: String,
// pub version: Version,
// pub name: String,
// pub data: Payload,
// pub metadata: Option<Meta>,
// pub created_utc: DateTime<Utc>,

// Event Stream
// pub id: String,
// pub last_version: Version,
// pub last_updated_utc: DateTime<Utc>,

pub struct EventStoreSQLXPostgres {
    pool: PgPool,
    streams_table_name: String,
    events_table_name: String,
}

impl EventStoreSQLXPostgres {
    async fn create_stream_table(pool: &PgPool, streams_name: &str) -> Result<PgDone>  {

        // create table if not exists cs_stream_person (
        //     id text primary key,
        // last_version bigint not null ,
        // last_updated_utc timestamptz default current_timestamp
        // );
        let streams_create_table = format!(
            "create table if not exists \
                    {} (id text primary key, \
                    last_version bigint not null, \
                    last_updated_utc timestamptz default current_timestamp )",
            streams_name
        );

        let res = sqlx::query(&streams_create_table).execute(pool).await?;
        Ok(res)
    }

    async fn create_event_table(pool: &PgPool, events_name: &str, streams_name: &str) -> Result<PgDone> {
        // create table if not exists cs_events_person (
        //     id uuid primary key,
        // correlation_id uuid default null,
        // causation_id uuid default null,
        // stream_id text not null,
        // constraint fk_stream foreign key (stream_id) references cs_stream_person(id) on delete cascade,
        // version bigint not null,
        // name varchar(255) not null ,
        // data jsonb not null ,
        // metadata jsonb,
        // created_utc timestamptz default current_timestamp
        // );
        let events_create_table = format!(
            "create table if not exists {0} (\
                    id uuid primary key,\
                    correlation_id uuid default null,\
                    causation_id uuid default null,\
                    stream_id text not null,\
                    constraint fk_stream foreign key (stream_id) references {1}(id) \
                    on delete cascade,\
                    version bigint not null,\
                    name varchar(255) not null ,\
                    data jsonb not null ,\
                    metadata jsonb,\
                    created_utc timestamptz default current_timestamp)",
            events_name, streams_name
        );

        let res = sqlx::query(&events_create_table).execute(pool).await?;
        Ok(res)
    }

    async fn create_timestamp_trigger(pool: &PgPool, streams_name: &str) -> Result<PgDone> {
        let trigger_function = format!(
            "create or replace function update_modified_column()
            returns trigger as $$
            begin
                new.last_updated_utc = current_timestamp;
                return new;
            end;
            $$ language 'plpgsql';"
        );

        let drop_trigger = format!("drop trigger if exists update_{0} on {0}", streams_name);

        let create_trigger =
            format!("create trigger update_{0} before update on {0} \
                    for each row execute procedure update_modified_column()", streams_name);

        let _ = sqlx::query(&trigger_function).execute(pool).await?;
        let _ = sqlx::query(&drop_trigger).execute(pool).await?;
        let res = sqlx::query(&create_trigger).execute(pool).await?;
        Ok(res)

    }


    pub async fn new(pool: &PgPool, name: &str) -> Result<EventStoreSQLXPostgres> {
        // Generate stream name
        let streams_name = format!("cv_streams_{}", name);
        // Generate name for event table
        let events_name = format!("cv_events_{}", name);

        let _ = EventStoreSQLXPostgres::create_stream_table(pool, &streams_name).await?;
        let _ = EventStoreSQLXPostgres::create_event_table(pool, &events_name, &streams_name).await?;
        let _ = EventStoreSQLXPostgres::create_timestamp_trigger(pool, &streams_name).await?;

        Ok(EventStoreSQLXPostgres {
            pool: pool.clone(),
            streams_table_name: streams_name,
            events_table_name: events_name,
        })
    }

    async fn process_events<Payload: Clone, Meta: Clone>(
        &mut self,
        stream_id: &str,
        version: &ExpectedVersion<EventVersion>,
        payload: Vec<EventWrite<Payload, Meta>>,
    ) -> Result<Vec<EventRead<Payload, Meta, EventVersion>>> {
        let exist =
            sqlx::query_as::<_, DBEventStream>("select * from cs_stream_person where id = ? limit 1;")
            .bind(stream_id).fetch_one(&self.pool).await;
        let last = match exist {
            Ok(r) => (r.last_version.clone(), Some(r)),
            Err(_) => (EventVersion::new(0), None),
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

        // Updating all in single transection
        let mut tr = self.pool.begin().await?;

        let _ = sqlx::query("insert into cs_stream_person (id, last_version)
values (?, ?) on conflict (id) do update set last_version = ?")
            .bind(updated_stream.id)
            .bind(updated_stream.last_version.0)
            .execute(&mut tr)
            .await?;

        tr.commit().await?;

        Ok(ops)
    }
}

#[async_trait]
impl<Payload: Clone, Meta: Clone> EventStore<Payload, Meta, EventVersion> for EventStoreSQLXPostgres
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
        unimplemented!()
    }

    async fn append_events(
        &mut self,
        stream_id: &str,
        version: &ExpectedVersion<EventVersion>,
        payload: Vec<EventWrite<Payload, Meta>>,
    ) -> Result<Vec<EventRead<Payload, Meta, EventVersion>>> {
        unimplemented!()
    }

    async fn get_event(
        &self,
        stream_id: &str,
        version: &EventVersion,
    ) -> Result<EventRead<Payload, Meta, EventVersion>> {
        unimplemented!()
    }

    async fn get_events(
        &self,
        stream_id: &str,
        version: &EventsReadRange<EventVersion>,
    ) -> Result<Vec<EventRead<Payload, Meta, EventVersion>>> {
        unimplemented!()
    }

    async fn get_events_by_correlation_id(
        &self,
        correlation_id: &Uuid,
    ) -> Result<Vec<EventRead<Payload, Meta, EventVersion>>> {
        unimplemented!()
    }

    async fn get_streams(
        &self,
        filter: &StreamsReadFilter,
    ) -> Result<Vec<EventStream<EventVersion>>> {
        unimplemented!()
    }

    async fn get_stream(&self, stream_id: &str) -> Result<EventStream<EventVersion>> {
        unimplemented!()
    }
}
