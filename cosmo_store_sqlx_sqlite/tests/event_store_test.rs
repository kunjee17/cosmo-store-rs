#[cfg(test)]
#[macro_use]
extern crate claim;

use cosmo_store::common::i64_event_version::EventVersion;
use cosmo_store::traits::event_store::EventStore;
use cosmo_store::types::event_read::EventRead;
use cosmo_store::types::expected_version::ExpectedVersion;
use cosmo_store_sqlx_sqlite::event_store_sqlx_sqlite::EventStoreSQLXSqlite;
use cosmo_store_tests::event_store_basic_tests as bt;
use cosmo_store_tests::event_store_basic_tests::{check_position, Meta, Payload};

use futures::{FutureExt, StreamExt};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePoolOptions;
use std::panic;
use uuid::Uuid;

const CONN_BASE: &str = "sqlite::memory:";

async fn setup() {
    println!("Event Store will be initialized here...");
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

async fn get_store<Payload, Meta>() -> impl EventStore<Payload, Meta, EventVersion>
where
    Payload: Send + Sync + 'static + Clone + Serialize + for<'de> Deserialize<'de>,
    Meta: Send + Sync + 'static + Clone + Serialize + for<'de> Deserialize<'de>,
{
    let conn_str = format!("{}", CONN_BASE);
    let pool = SqlitePoolOptions::new().connect(&conn_str).await.unwrap();
    let store = EventStoreSQLXSqlite::new(&pool, "person").await.unwrap();
    store
}

// fn get_name() -> String {
//     Uuid::new_v4().as_simple().to_string()
// }

// TODO: implement to improve test code. not working as of now
// async fn run_test<T: Sized>(test: Box<T>) -> ()
//     where
//         T: Fn(&str) -> dyn Future<Output=()>,
// {
//     let name = Uuid::new_v4().to_simple().to_string();
//     setup().await;
//
//     let result = panic::AssertUnwindSafe(test(&name).await).catch_unwind().await;
//     test(&name).await;
//     teardown().await;
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
async fn store_setup() {
    let store = get_store::<Payload, Meta>().await;
    assert!(true);
}

#[actix_rt::test]
async fn append_event() {
    setup().await;
    let result = std::panic::AssertUnwindSafe(bt::append_event(&get_store().await, |res| {
        assert_eq!(res.version.0, 1_i64)
    }))
    .catch_unwind()
    .await;

    teardown().await;

    assert_ok!(result);
}

#[actix_rt::test]
async fn append_100_events() {
    setup().await;
    let result = std::panic::AssertUnwindSafe(bt::append_100_events(&get_store().await, |res| {
        assert_eq!(res.len(), 100);
        are_ascending(res);
    }))
    .catch_unwind()
    .await;
    teardown().await;

    assert_ok!(result);
}

#[actix_rt::test]
async fn get_single_event() {
    setup().await;
    let result = std::panic::AssertUnwindSafe(bt::get_single_event(
        &get_store().await,
        &EventVersion::new(3_i64),
        |res| {
            assert_eq!(res.version.0, 3_i64);
            assert_eq!(res.name, "Created_3");
        },
    ))
    .catch_unwind()
    .await;

    teardown().await;

    assert_ok!(result);
}

#[actix_rt::test]
async fn get_all_events() {
    setup().await;
    let result = std::panic::AssertUnwindSafe(bt::get_all_events(&get_store().await, |res| {
        assert_eq!(res.len(), 10);
    }))
    .catch_unwind()
    .await;

    teardown().await;

    assert_ok!(result);
}

#[actix_rt::test]
async fn get_events_from_version() {
    setup().await;
    let result = std::panic::AssertUnwindSafe(bt::get_events_from_version(
        &get_store().await,
        EventVersion::new(6),
        |res| {
            assert_eq!(res.len(), 5);
            are_ascending(res);
        },
    ))
    .catch_unwind()
    .await;

    teardown().await;

    assert_ok!(result);
}

#[actix_rt::test]
async fn get_events_to_version() {
    setup().await;
    let result = std::panic::AssertUnwindSafe(bt::get_events_to_version(
        &get_store().await,
        EventVersion::new(5),
        |res| {
            assert_eq!(res.len(), 5);
            are_ascending(res);
        },
    ))
    .catch_unwind()
    .await;

    teardown().await;

    assert_ok!(result);
}

#[actix_rt::test]
async fn get_events_version_range() {
    setup().await;
    let result = std::panic::AssertUnwindSafe(bt::get_events_version_range(
        &get_store().await,
        EventVersion::new(5),
        EventVersion::new(7),
        |res| {
            assert_eq!(res.len(), 3);
            are_ascending(res);
        },
    ))
    .catch_unwind()
    .await;

    teardown().await;

    assert_ok!(result);
}

#[actix_rt::test]
async fn fails_to_append_to_existing_version() {
    setup().await;
    let result = std::panic::AssertUnwindSafe(bt::fails_to_append_to_existing_version(
        &get_store().await,
        EventVersion::new(1_i64),
        |res| {
            assert_err!(res);
        },
    ))
    .catch_unwind()
    .await;
    teardown().await;

    assert_ok!(result);
}

#[actix_rt::test]
async fn fails_to_append_to_existing_stream_if_is_not_expected_to_exist() {
    setup().await;
    let result = std::panic::AssertUnwindSafe(
        bt::fails_to_append_to_existing_stream_if_is_not_expected_to_exist(
            &get_store().await,
            |res| {
                assert_err!(res);
            },
        ),
    )
    .catch_unwind()
    .await;
    teardown().await;

    assert_ok!(result);
}

#[actix_rt::test]
async fn appending_no_events_does_not_affect_stream_metadata() {
    setup().await;
    let result =
        std::panic::AssertUnwindSafe(bt::appending_no_events_does_not_affect_stream_metadata(
            &get_store().await,
            &ExpectedVersion::Exact(EventVersion::new(1_i64)),
            |stream, stream_after_append| {
                assert_eq!(stream, stream_after_append);
            },
        ))
        .catch_unwind()
        .await;
    teardown().await;

    assert_ok!(result);
}

#[actix_rt::test]
async fn appending_1000_events_can_be_read_back() {
    setup().await;
    let result = std::panic::AssertUnwindSafe(bt::appending_1000_events_can_be_read_back::<
        EventVersion,
        _,
    >(&get_store().await, |stream, events| {
        assert_eq!(stream.last_version.0, 1000);
        assert_eq!(events.len(), 1000)
    }))
    .catch_unwind()
    .await;
    teardown().await;

    assert_ok!(result);
}

#[actix_rt::test]
async fn can_read_events_by_correlation_id() {
    setup().await;
    let result = std::panic::AssertUnwindSafe(bt::can_read_events_by_correlation_id::<
        EventVersion,
        _,
    >(&get_store().await, |events| {
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
    teardown().await;

    assert_ok!(result);
}
