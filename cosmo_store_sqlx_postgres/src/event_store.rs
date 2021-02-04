use crate::event_version::EventVersion;
use anyhow::{bail, Result};
use async_trait::async_trait;
use cosmo_store::{
    EventRead, EventStore, EventStream, EventWrite, EventsReadRange, ExpectedVersion,
    StreamsReadFilter,
};
use sqlx::types::Uuid;
use sqlx::{PgPool, Acquire};
use std::borrow::Borrow;

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
    pub async fn new(pool: &PgPool, name: &str) -> Result<EventStoreSQLXPostgres> {
        // Check if table exist and if not create them
        let streams_name = format!("cv_streams_{}", name);
        let events_name = format!("cv_events_{}", name);
        // create table if not exists cs_stream_person (
        //     id text primary key,
        // last_version bigint not null ,
        // last_updated_utc timestamptz default current_timestamp
        // );
        let streams_create = format!(
            "create table if not exists \
                    {} (id text primary key, \
                    last_version bigint not null, \
                    last_updated_utc timestamptz default current_timestamp )",
            streams_name
        );

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

        let events_create = format!(
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

        let trigger_function = format!(
            "create or replace function update_modified_column()
            returns trigger as $$
            begin
                new.last_updated_utc = current_timestamp;
                return new;
            end;
            $$ language 'plpgsql';"
        );

        let create_trigger = format!("create trigger update_{0} before update on {0} for each row execute procedure update_modified_column()", streams_name);

        let _ = sqlx::query(&streams_create).execute(pool).await?;
        let _ = sqlx::query(&events_create).execute(pool).await?;
        let _ = sqlx::query(&trigger_function).execute(pool).await?;
        let _ = sqlx::query(&create_trigger).execute(pool).await?;

        println!("Stream, Event, trigger function and create trigger created");

        Ok(EventStoreSQLXPostgres {
            pool: pool.clone(),
            streams_table_name: name.to_string(),
            events_table_name: name.to_string(),
        })
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
