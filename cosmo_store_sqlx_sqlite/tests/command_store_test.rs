#[cfg(test)]
#[macro_use]
extern crate claim;

use cosmo_store::traits::command_store::CommandStore;
use cosmo_store::types::command_write::CommandWrite;
use cosmo_store_sqlx_sqlite::command_store_sqlx_sqlite::CommandStoreSQLXSqlite;
use futures::FutureExt;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePoolOptions;
use uuid::Uuid;

const CONN_BASE: &str = "sqlite::memory:";

async fn setup() {
    println!("Command Store will be initialized here...");
    let conn_str = CONN_BASE.to_string();
    let pool = SqlitePoolOptions::new().connect(&conn_str).await.unwrap();
    // let create_db = format!("create database \"{}\" encoding = 'UTF8'", name);
    // let _ = sqlx::query(&create_db).execute(&pool).await.unwrap();
    println!("Created {}", !pool.is_closed());
}

async fn teardown() {
    println!("Event Store will be destroyed here...");
    // let conn_str = CONN_BASE.to_string();
    // let pool = SqlitePoolOptions::new().connect(&conn_str).await.unwrap();
    // let kill_conn = format!(
    //     "select pg_terminate_backend(pid) from pg_stat_activity where datname='{}'",
    //     name
    // );
    // let create_db = format!("drop database if exists \"{}\"", name);
    // let _ = sqlx::query(&kill_conn).execute(&pool).await.unwrap();
    // let _ = sqlx::query(&create_db).execute(&pool).await.unwrap();
    println!("Destroyed");
}

async fn get_store<Payload>() -> impl CommandStore<Payload>
where
    Payload: Send + Sync + 'static + Clone + Serialize + for<'de> Deserialize<'de>,
{
    let conn_str = format!("{}", CONN_BASE);
    let pool = SqlitePoolOptions::new().connect(&conn_str).await.unwrap();
    let store = CommandStoreSQLXSqlite::new(&pool, "person").await.unwrap();
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
    setup().await;
    let store = get_store().await;
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

    teardown().await;

    assert_ok!(result);
}
