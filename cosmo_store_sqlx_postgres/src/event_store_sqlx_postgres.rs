use anyhow::Result;
use sqlx::postgres::PgQueryResult;
use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct EventStoreSQLXPostgres {
    pool: PgPool,
    streams_table_name: String,
    events_table_name: String,
}

impl EventStoreSQLXPostgres {
    pub fn pool(&self) -> PgPool {
        self.pool.clone()
    }

    pub fn streams_table_name(&self) -> String {
        self.streams_table_name.to_string()
    }

    pub fn events_table_name(&self) -> String {
        self.events_table_name.to_string()
    }

    async fn create_stream_table(pool: &PgPool, streams_name: &str) -> Result<PgQueryResult> {
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

    async fn create_event_table(
        pool: &PgPool,
        events_name: &str,
        streams_name: &str,
    ) -> Result<PgQueryResult> {
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
                    metadata jsonb default null,\
                    created_utc timestamptz default current_timestamp)",
            events_name, streams_name
        );

        let res = sqlx::query(&events_create_table).execute(pool).await?;
        Ok(res)
    }

    async fn create_timestamp_trigger(pool: &PgPool, streams_name: &str) -> Result<PgQueryResult> {
        let trigger_function = r#"create or replace function update_modified_column()
            returns trigger as $$
            begin
                new.last_updated_utc = current_timestamp;
                return new;
            end;
            $$ language 'plpgsql';"#;


        let create_trigger = format!(
            "create trigger if not exists update_{0} before update on {0} \
                    for each row execute procedure update_modified_column()",
            streams_name
        );

        let _ = sqlx::query(&trigger_function).execute(pool).await?;
        let res = sqlx::query(&create_trigger).execute(pool).await?;
        Ok(res)
    }

    pub async fn new(pool: &PgPool, name: &str) -> Result<EventStoreSQLXPostgres> {
        // Generate stream name
        let streams_name = format!("cs_streams_{}", name);
        // Generate name for event table
        let events_name = format!("cs_events_{}", name);

        let _ = EventStoreSQLXPostgres::create_stream_table(pool, &streams_name).await?;
        let _ =
            EventStoreSQLXPostgres::create_event_table(pool, &events_name, &streams_name).await?;
        let _ = EventStoreSQLXPostgres::create_timestamp_trigger(pool, &streams_name).await?;

        Ok(EventStoreSQLXPostgres {
            pool: pool.clone(),
            streams_table_name: streams_name,
            events_table_name: events_name,
        })
    }
}
