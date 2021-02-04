use cosmo_store_tests::event_generator::{get_event, get_stream_id};
use cosmo_store_tests::event_store_basic_tests as bt;
use cosmo_store_tests::event_store_basic_tests::{check_position, Meta, Payload};
use cosmo_store::{EventStore, EventRead};
use anyhow::Result;
use cosmo_store_sqlx_postgres::event_version::EventVersion;
use cosmo_store_sqlx_postgres::event_store::EventStoreSQLXPostgres;
use sqlx::postgres::PgPoolOptions;
use std::panic;
use uuid::Uuid;
use sqlx::Done;
use futures;
use futures::{FutureExt, Future};
use std::any::Any;

const CONN_BASE: &str = "postgresql://localhost:5432/";

async fn setup(name : &str) {
    println!("Event Store will be initialized here...");
    let conn_str = format!("{}", CONN_BASE);
    let pool = PgPoolOptions::new().connect(&conn_str).await.unwrap();
    let create_db = format!("create database \"{}\" encoding = 'UTF8'", name);
    let _ = sqlx::query(&create_db).execute(&pool).await.unwrap();
    println!("Created {}", name);
}

async fn teardown(name : &str) {
    println!("Event Store will be destroyed here...");
    let conn_str = format!("{}", CONN_BASE);
    let pool = PgPoolOptions::new().connect(&conn_str).await.unwrap();
    let kill_conn = format!("select pg_terminate_backend(pid) from pg_stat_activity where datname='{}'", name);
    let create_db = format!("drop database if exists \"{}\"", name);
    let _ = sqlx::query(&kill_conn).execute(&pool).await.unwrap();
    let _ = sqlx::query(&create_db).execute(&pool).await.unwrap();
    println!("Destroyed {}", name);
}


async fn get_store(name : &str) -> impl EventStore<Payload, Meta, EventVersion> {
    let conn_str = format!("{}{}", CONN_BASE, name);
    let pool = PgPoolOptions::new().connect(&conn_str).await.unwrap();
    let store = EventStoreSQLXPostgres::new(&pool, "person").await.unwrap();
    store
}


fn get_name() -> String {
    Uuid::new_v4().to_simple().to_string()
}

// TODO: implement to improve test code. not working as of now
// async fn run_test<T>(test: T) -> ()
//     where
//         T: Fn(&str) -> dyn Future<Output=()>,
// {
//     let name = Uuid::new_v4().to_simple().to_string();
//     setup(&name).await;
//
//     let result = panic::AssertUnwindSafe(test(&name)).catch_unwind().await;
//     test(&name).await;
//     teardown(&name).await;
//     assert!(result.is_ok())
// }


#[actix_rt::test]
async fn hello_world_async() {
    assert_eq!(2 + 2, 4);
}

fn are_ascending(items: Vec<EventRead<Payload, Meta, EventVersion>>) {
    items.iter().fold(0_u32, |acc, item| {
        check_position(|| item.version.0 > acc, item.version.0)
    });
}

#[actix_rt::test]
async fn append_event() {
    let name = get_name();
    setup(&name).await;
    let result = std::panic::AssertUnwindSafe(
        bt::append_event(Box::new(get_store(&name).await), |res| {
            assert_eq!(res.version.0, 1_u32)
        })
    ).catch_unwind().await;

    teardown(&name).await;

    assert!(result.is_ok());
}

#[actix_rt::test]
async fn append_100_events() {
    let name = get_name();
    setup(&name).await;
    let result = std::panic::AssertUnwindSafe(
    bt::append_100_events(Box::new(get_store(&name).await), |res| {
        assert_eq!(res.len(), 100);
        are_ascending(res);
    })).catch_unwind().await;
    teardown(&name).await;

    assert!(result.is_ok());
}


#[actix_rt::test]
async fn get_single_event() {
    let name = get_name();
    setup(&name).await;
    let result = std::panic::AssertUnwindSafe(
    bt::get_single_event(Box::new(get_store(&name).await), &EventVersion::new(3_u32), |res| {
        assert_eq!(res.version.0, 3_u32);
        assert_eq!(res.name, "Created_3");
    })).catch_unwind().await;

    teardown(&name).await;

    assert!(result.is_ok());
}

#[actix_rt::test]
async fn get_all_events() {
    let name = get_name();
    setup(&name).await;
    let result = std::panic::AssertUnwindSafe(
    bt::get_all_events(Box::new(get_store(&name).await), |res| {
        assert_eq!(res.len(), 10);
    })).catch_unwind().await;

    teardown(&name).await;

    assert!(result.is_ok());
}

#[actix_rt::test]
async fn get_events_from_version() {
    let name = get_name();
    setup(&name).await;
    let result = std::panic::AssertUnwindSafe(
    bt::get_events_from_version(Box::new(get_store(&name).await), EventVersion::new(6), |res| {
        assert_eq!(res.len(), 5);
        are_ascending(res);
    })).catch_unwind().await;

    teardown(&name).await;

    assert!(result.is_ok());
}

#[actix_rt::test]
async fn get_events_to_version() {
    let name = get_name();
    setup(&name).await;
    let result = std::panic::AssertUnwindSafe(
    bt::get_events_to_version(Box::new(get_store(&name).await), EventVersion::new(5), |res| {
        assert_eq!(res.len(), 5);
        are_ascending(res);
    })).catch_unwind().await;

    teardown(&name).await;

    assert!(result.is_ok());
}

#[actix_rt::test]
async fn get_events_version_range() {
    let name = get_name();
    setup(&name).await;
    let result = std::panic::AssertUnwindSafe(
    bt::get_events_version_range(
        Box::new(get_store(&name).await),
        EventVersion::new(5),
        EventVersion::new(7),
        |res| {
            assert_eq!(res.len(), 3);
            are_ascending(res);
        },
    )).catch_unwind().await;

    teardown(&name).await;

    assert!(result.is_ok());
}

