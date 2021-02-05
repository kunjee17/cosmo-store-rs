#[cfg(test)]
#[macro_use]
extern crate claim;

use cosmo_store::common::i64_event_version::EventVersion;
use cosmo_store::traits::event_store::EventStore;
use cosmo_store::types::event_read::EventRead;
use cosmo_store::types::expected_version::ExpectedVersion;
use cosmo_store_sqlx_postgres::event_store_sqlx_postgres::EventStoreSQLXPostgres;
use cosmo_store_tests::event_store_basic_tests as bt;
use cosmo_store_tests::event_store_basic_tests::{check_position, Meta, Payload};
use futures;
use futures::FutureExt;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use std::panic;
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

async fn get_store<Payload, Meta>(name: &str) -> impl EventStore<Payload, Meta, EventVersion>
where
    Payload: Send + Sync + 'static + Clone + Serialize + for<'de> Deserialize<'de>,
    Meta: Send + Sync + 'static + Clone + Serialize + for<'de> Deserialize<'de>,
{
    let conn_str = format!("{}{}", CONN_BASE, name);
    let pool = PgPoolOptions::new().connect(&conn_str).await.unwrap();
    let store = EventStoreSQLXPostgres::new(&pool, "person").await.unwrap();
    store
}

fn get_name() -> String {
    Uuid::new_v4().to_simple().to_string()
}

// TODO: implement to improve test code. not working as of now
// async fn run_test<T: Sized>(test: Box<T>) -> ()
//     where
//         T: Fn(&str) -> dyn Future<Output=()>,
// {
//     let name = Uuid::new_v4().to_simple().to_string();
//     setup(&name).await;
//
//     let result = panic::AssertUnwindSafe(test(&name).await).catch_unwind().await;
//     test(&name).await;
//     teardown(&name).await;
//     assert!(result.is_ok())
// }

#[actix_rt::test]
async fn hello_world_async() {
    assert_eq!(2 + 2, 4);
}

fn are_ascending(items: Vec<EventRead<Payload, Meta, EventVersion>>) {
    items.iter().fold(0_i64, |acc, item| {
        check_position(|| item.version.0 > acc, item.version.0)
    });
}

#[actix_rt::test]
async fn append_event() {
    let name = get_name();
    setup(&name).await;
    let result =
        std::panic::AssertUnwindSafe(bt::append_event(Box::new(get_store(&name).await), |res| {
            assert_eq!(res.version.0, 1_i64)
        }))
        .catch_unwind()
        .await;

    teardown(&name).await;

    assert_ok!(result);
}

#[actix_rt::test]
async fn append_100_events() {
    let name = get_name();
    setup(&name).await;
    let result = std::panic::AssertUnwindSafe(bt::append_100_events(
        Box::new(get_store(&name).await),
        |res| {
            assert_eq!(res.len(), 100);
            are_ascending(res);
        },
    ))
    .catch_unwind()
    .await;
    teardown(&name).await;

    assert_ok!(result);
}

#[actix_rt::test]
async fn get_single_event() {
    let name = get_name();
    setup(&name).await;
    let result = std::panic::AssertUnwindSafe(bt::get_single_event(
        Box::new(get_store(&name).await),
        &EventVersion::new(3_i64),
        |res| {
            assert_eq!(res.version.0, 3_i64);
            assert_eq!(res.name, "Created_3");
        },
    ))
    .catch_unwind()
    .await;

    teardown(&name).await;

    assert_ok!(result);
}

#[actix_rt::test]
async fn get_all_events() {
    let name = get_name();
    setup(&name).await;
    let result = std::panic::AssertUnwindSafe(bt::get_all_events(
        Box::new(get_store(&name).await),
        |res| {
            assert_eq!(res.len(), 10);
        },
    ))
    .catch_unwind()
    .await;

    teardown(&name).await;

    assert_ok!(result);
}

#[actix_rt::test]
async fn get_events_from_version() {
    let name = get_name();
    setup(&name).await;
    let result = std::panic::AssertUnwindSafe(bt::get_events_from_version(
        Box::new(get_store(&name).await),
        EventVersion::new(6),
        |res| {
            assert_eq!(res.len(), 5);
            are_ascending(res);
        },
    ))
    .catch_unwind()
    .await;

    teardown(&name).await;

    assert_ok!(result);
}

#[actix_rt::test]
async fn get_events_to_version() {
    let name = get_name();
    setup(&name).await;
    let result = std::panic::AssertUnwindSafe(bt::get_events_to_version(
        Box::new(get_store(&name).await),
        EventVersion::new(5),
        |res| {
            assert_eq!(res.len(), 5);
            are_ascending(res);
        },
    ))
    .catch_unwind()
    .await;

    teardown(&name).await;

    assert_ok!(result);
}

#[actix_rt::test]
async fn get_events_version_range() {
    let name = get_name();
    setup(&name).await;
    let result = std::panic::AssertUnwindSafe(bt::get_events_version_range(
        Box::new(get_store(&name).await),
        EventVersion::new(5),
        EventVersion::new(7),
        |res| {
            assert_eq!(res.len(), 3);
            are_ascending(res);
        },
    ))
    .catch_unwind()
    .await;

    teardown(&name).await;

    assert_ok!(result);
}

#[actix_rt::test]
async fn fails_to_append_to_existing_version() {
    let name = get_name();
    setup(&name).await;
    let result = std::panic::AssertUnwindSafe(bt::fails_to_append_to_existing_version(
        Box::new(get_store(&name).await),
        EventVersion::new(1_i64),
        |res| {
            assert_err!(res);
        },
    ))
    .catch_unwind()
    .await;
    teardown(&name).await;

    assert_ok!(result);
}

#[actix_rt::test]
async fn fails_to_append_to_existing_stream_if_is_not_expected_to_exist() {
    let name = get_name();
    setup(&name).await;
    let result = std::panic::AssertUnwindSafe(
        bt::fails_to_append_to_existing_stream_if_is_not_expected_to_exist(
            Box::new(get_store(&name).await),
            |res| {
                assert_err!(res);
            },
        ),
    )
    .catch_unwind()
    .await;
    teardown(&name).await;

    assert_ok!(result);
}

#[actix_rt::test]
async fn appending_no_events_does_not_affect_stream_metadata() {
    let name = get_name();
    setup(&name).await;
    let result =
        std::panic::AssertUnwindSafe(bt::appending_no_events_does_not_affect_stream_metadata(
            Box::new(get_store(&name).await),
            &ExpectedVersion::Exact(EventVersion::new(1_i64)),
            |stream, stream_after_append| {
                assert_eq!(stream, stream_after_append);
            },
        ))
        .catch_unwind()
        .await;
    teardown(&name).await;

    assert_ok!(result);
}

#[actix_rt::test]
async fn appending_1000_events_can_be_read_back() {
    let name = get_name();
    setup(&name).await;
    let result = std::panic::AssertUnwindSafe(bt::appending_1000_events_can_be_read_back::<
        EventVersion,
        _,
    >(
        Box::new(get_store(&name).await),
        |stream, events| {
            assert_eq!(stream.last_version.0, 1000);
            assert_eq!(events.len(), 1000)
        },
    ))
    .catch_unwind()
    .await;
    teardown(&name).await;

    assert_ok!(result);
}

#[actix_rt::test]
async fn can_read_events_by_correlation_id() {
    let name = get_name();
    setup(&name).await;
    let result = std::panic::AssertUnwindSafe(bt::can_read_events_by_correlation_id::<
        EventVersion,
        _,
    >(Box::new(get_store(&name).await), |events| {
        let unique_streams: Vec<String> = events
            .iter()
            .map(|x| x.stream_id.to_owned())
            .unique()
            .sorted()
            .collect();
        assert_eq!(unique_streams, vec!["CORR_1", "CORR_2", "CORR_3"]);
        assert_eq!(events.len(), 30);
    }))
    .catch_unwind()
    .await;
    teardown(&name).await;

    assert_ok!(result);
}
