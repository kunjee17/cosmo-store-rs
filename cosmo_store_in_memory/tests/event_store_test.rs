use cosmo_store::{EventRead, EventStore, EventWrite, EventsReadRange, ExpectedVersion};
use cosmo_store_in_memory::event_store::EventStoreInMemory;
use cosmo_store_in_memory::event_version::EventVersion;
use cosmo_store_tests::event_generator::{get_event, get_stream_id};
use cosmo_store_tests::event_store_basic_tests as bt;
use cosmo_store_tests::event_store_basic_tests::{check_position, Meta, Payload};
use std::collections::HashMap;
use std::future::Future;
use std::panic;

fn setup() {
    println!("Event Store will be initialized here...");
}

fn teardown() {
    println!("Event Store will be destroyed here");
}

fn get_store() -> impl EventStore<Payload, Meta, EventVersion> {
    EventStoreInMemory::new()
}

fn run_test<T>(test: T) -> ()
where
    T: FnOnce(&dyn EventStore<Payload, Meta, EventVersion>) -> () + panic::UnwindSafe,
{
    // setup();
    let store: EventStoreInMemory<Payload, Meta, EventVersion> = EventStoreInMemory::new();
    let result = panic::catch_unwind(|| test(&store));
    // teardown();
    assert!(result.is_ok())
}

#[test]
fn hello_world() {
    run_test(|_| {
        assert_eq!(2 + 2, 4);
    })
}

fn are_ascending(items: Vec<EventRead<Payload, Meta, EventVersion>>) {
    items.iter().fold(0_u32, |acc, item| {
        check_position(|| item.version.0 > acc, item.version.0)
    });
}

#[actix_rt::test]
async fn append_event() {
    bt::append_event(Box::new(get_store()), |res| {
        assert_eq!(res.version.0, 1_u32)
    })
    .await;
}

#[actix_rt::test]
async fn append_100_events() {
    bt::append_100_events(Box::new(get_store()), |res| {
        assert_eq!(res.len(), 100);
        are_ascending(res);
    })
    .await;
}

#[actix_rt::test]
async fn get_single_event() {
    bt::get_single_event(Box::new(get_store()), &EventVersion::new(3_u32), |res| {
        assert_eq!(res.version.0, 3_u32);
        assert_eq!(res.name, "Created_3");
    })
    .await;
}

#[actix_rt::test]
async fn get_all_events() {
    bt::get_all_events(Box::new(get_store()), |res| {
        assert_eq!(res.len(), 10);
    })
    .await;
}

#[actix_rt::test]
async fn get_events_from_version() {
    bt::get_events_from_version(Box::new(get_store()), EventVersion::new(6), |res| {
        assert_eq!(res.len(), 5);
        are_ascending(res);
    })
    .await;
}

#[actix_rt::test]
async fn get_events_to_version() {
    bt::get_events_to_version(Box::new(get_store()), EventVersion::new(5), |res| {
        assert_eq!(res.len(), 5);
        are_ascending(res);
    })
    .await;
}

#[actix_rt::test]
async fn get_events_version_range() {
    bt::get_events_version_range(
        Box::new(get_store()),
        EventVersion::new(5),
        EventVersion::new(7),
        |res| {
            assert_eq!(res.len(), 3);
            are_ascending(res);
        },
    )
    .await;
}
