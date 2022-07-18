use anyhow::Result;
use sqlx::sqlite::SqlitePool;
use sqlx::sqlite::SqliteQueryResult;

#[derive(Debug, Clone)]
pub struct EventStoreSQLXSqlite {
    pool: SqlitePool,
    streams_table_name: String,
    events_table_name: String,
}

impl EventStoreSQLXSqlite {
    pub fn pool(&self) -> SqlitePool {
        self.pool.clone()
    }

    pub fn streams_table_name(&self) -> String {
        self.streams_table_name.to_string()
    }

    pub fn events_table_name(&self) -> String {
        self.events_table_name.to_string()
    }

    async fn create_stream_table(
        pool: &SqlitePool,
        streams_name: &str,
    ) -> Result<SqliteQueryResult> {
        // create table if not exists cs_stream_person (
        //     id text primary key,
        // last_version bigint not null ,
        // last_updated_utc timestamptz default current_timestamp
        // );
        let streams_create_table = format!(
            "create table if not exists \
                    {0} (id text primary key, \
                    last_version integer not null, \
                    last_updated_utc date default (datetime('now','utc')))",
            streams_name
        );

        let res = sqlx::query(&streams_create_table).execute(pool).await?;
        Ok(res)
    }

    async fn create_event_table(
        pool: &SqlitePool,
        events_name: &str,
        streams_name: &str,
    ) -> Result<SqliteQueryResult> {
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
                    id text primary key,\
                    correlation_id text default null,\
                    causation_id text default null,\
                    stream_id text not null,\
                    version integer,\
                    name varchar(255) not null ,\
                    data json not null ,\
                    metadata json default null,\
                    created_utc date default (datetime('now','utc')),\
                    constraint fk_stream foreign key (stream_id) references {1}(id) \
                    on delete cascade)\
                    ",
            events_name, streams_name
        );

        let res = sqlx::query(&events_create_table).execute(pool).await?;
        Ok(res)
    }

    async fn create_timestamp_trigger(
        pool: &SqlitePool,
        streams_name: &str,
    ) -> Result<SqliteQueryResult> {
        // let trigger_function = r#"create or replace function update_modified_column()
        //     returns trigger as $$
        //     begin
        //         new.last_updated_utc = current_timestamp;
        //         return new;
        //     end;
        //     $$ language 'plpgsql';"#;

        let drop_trigger = format!("drop trigger if exists update_{0}", streams_name);

        let create_trigger = format!(
            "create trigger update_{0} after update on {0} \
             begin \
                update {0} set last_updated_utc = datetime('now', 'utc') where id = new.id;
             end;\
             ",
            streams_name
        );

        // let _ = sqlx::query(trigger_function).execute(pool).await?;
        let _ = sqlx::query(&drop_trigger).execute(pool).await?;
        let res = sqlx::query(&create_trigger).execute(pool).await?;
        Ok(res)
    }

    pub async fn new(pool: &SqlitePool, name: &str) -> Result<EventStoreSQLXSqlite> {
        // Generate stream name
        let streams_name = format!("cs_streams_{}", name);
        // Generate name for event table
        let events_name = format!("cs_events_{}", name);

        let _ = EventStoreSQLXSqlite::create_stream_table(pool, &streams_name).await?;
        let _ = EventStoreSQLXSqlite::create_event_table(pool, &events_name, &streams_name).await?;
        let _ = EventStoreSQLXSqlite::create_timestamp_trigger(pool, &streams_name).await?;

        Ok(EventStoreSQLXSqlite {
            pool: pool.clone(),
            streams_table_name: streams_name,
            events_table_name: events_name,
        })
    }
}
