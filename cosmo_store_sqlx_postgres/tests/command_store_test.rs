#[cfg(test)]
#[macro_use]
extern crate claim;

use chrono::Utc;
use cosmo_store::traits::command_store::CommandStore;
use cosmo_store::types::command_write::CommandWrite;
use cosmo_store_sqlx_postgres::command_store_sqlx_postgres::CommandStoreSQLXPostgres;
use futures::FutureExt;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

const CONN_BASE: &str = "postgresql://localhost:5432/";

async fn setup(name: &str) {
    println!("Event Store will be initialized here...");
    let conn_str = format!("{}", CONN_BASE);
    let pool = PgPoolOptions::new().connect(&conn_str).await.unwrap();
    let create_db = format!("create database \"{}\" encoding = 'UTF8'", name);
    let _ = sqlx::query(&create_db).execute(&pool).await.unwrap();
    println!("Created {}", name);
}

async fn teardown(name: &str) {
    println!("Event Store will be destroyed here...");
    let conn_str = format!("{}", CONN_BASE);
    let pool = PgPoolOptions::new().connect(&conn_str).await.unwrap();
    let kill_conn = format!(
        "select pg_terminate_backend(pid) from pg_stat_activity where datname='{}'",
        name
    );
    let create_db = format!("drop database if exists \"{}\"", name);
    let _ = sqlx::query(&kill_conn).execute(&pool).await.unwrap();
    let _ = sqlx::query(&create_db).execute(&pool).await.unwrap();
    println!("Destroyed {}", name);
}

async fn get_store<Payload>(name: &str) -> impl CommandStore<Payload>
where
    Payload: Send + Sync + 'static + Clone + Serialize + for<'de> Deserialize<'de>,
{
    let conn_str = format!("{}{}", CONN_BASE, name);
    let pool = PgPoolOptions::new().connect(&conn_str).await.unwrap();
    let store = CommandStoreSQLXPostgres::new(&pool, "person")
        .await
        .unwrap();
    store
}

fn get_name() -> String {
    Uuid::new_v4().as_simple().to_string()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct DummyCommand {
    text: String,
}

#[actix_rt::test]
async fn append_command() {
    let name = get_name();
    setup(&name).await;
    let store = get_store(&name).await;
    let id = Uuid::new_v4();
    let result = std::panic::AssertUnwindSafe(store.append_command(&CommandWrite {
        id,
        correlation_id: id,
        causation_id: id,
        data: DummyCommand {
            text: "Do Something".to_string(),
        },
        name: "some_command".to_string(),
    }))
    .catch_unwind()
    .await;

    teardown(&name).await;

    assert_ok!(result);
}
