use cosmo_store::{EventRead, EventStore, EventWrite, EventsReadRange, ExpectedVersion};
use cosmo_store_in_memory::event_store::EventStoreInMemory;
use cosmo_store_in_memory::event_version::EventVersion;
use cosmo_store_tests::event_generator::{get_event, get_stream_id};
use std::collections::HashMap;
use std::future::Future;
use std::panic;

// Keep the commented code for reference
// fn setup() {
//     println!("Event Store will be initialized here...");
// }

// fn teardown() {
//     println!("Event Store will be destroyed here");
// }

#[derive(Clone, Debug)]
struct Payload {
    name: String,
}
#[derive(Clone, Debug)]
struct Meta {}

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

fn check_position(acc: u32, item: &EventRead<Payload, Meta, EventVersion>) -> u32 {
    assert_eq!(item.version.0 > acc, true);
    item.version.0
}

fn are_ascending(items: Vec<EventRead<Payload, Meta, EventVersion>>) {
    items
        .iter()
        .fold(0_u32, |acc, item| check_position(acc, item));
}

#[test]
fn hello_world() {
    run_test(|_| {
        assert_eq!(2 + 2, 4);
    })
}

#[actix_rt::test]
async fn append_events() {
    let stream_id = get_stream_id();
    let event = get_event(
        0,
        Payload {
            name: "Todo Event".to_string(),
        },
    );
    let mut store = get_store();
    let res = store
        .append_event(&stream_id, &ExpectedVersion::Any, &event)
        .await
        .unwrap();
    assert_eq!(res.version.0, 1_u32);
}

#[actix_rt::test]
async fn append_100_events() {
    let stream_id = get_stream_id();
    let events: Vec<EventWrite<Payload, Meta>> = (1..=100)
        .map(|x| {
            get_event(
                x,
                Payload {
                    name: format!("Todo Event {}", x),
                },
            )
        })
        .collect();
    let mut store = get_store();
    let res: Vec<EventRead<Payload, Meta, EventVersion>> = store
        .append_events(&stream_id, &ExpectedVersion::Any, events)
        .await
        .unwrap();

    assert_eq!(res.len(), 100);
    are_ascending(res);
}

#[actix_rt::test]
async fn get_single_event() {
    let stream_id = get_stream_id();
    let events: Vec<EventWrite<Payload, Meta>> = (1..=10)
        .map(|x| {
            get_event(
                x,
                Payload {
                    name: format!("Todo Event {}", x),
                },
            )
        })
        .collect();
    let mut store = get_store();
    let _ = store
        .append_events(&stream_id, &ExpectedVersion::Any, events)
        .await
        .unwrap();

    let res = store
        .get_event(&stream_id, &EventVersion::new(3_u32))
        .await
        .unwrap();
    assert_eq!(res.version.0, 3_u32);
    assert_eq!(res.name, "Created_3");
}

#[actix_rt::test]
async fn get_all_events() {
    let stream_id = get_stream_id();
    let events: Vec<EventWrite<Payload, Meta>> = (1..=10)
        .map(|x| {
            get_event(
                x,
                Payload {
                    name: format!("Todo Event {}", x),
                },
            )
        })
        .collect();
    let mut store = get_store();
    let _ = store
        .append_events(&stream_id, &ExpectedVersion::Any, events)
        .await
        .unwrap();
    let res = store
        .get_events(&stream_id, &EventsReadRange::AllEvents)
        .await
        .unwrap();
    assert_eq!(res.len(), 10);
}

#[actix_rt::test]
async fn get_events_from_version() {
    let stream_id = get_stream_id();
    let events: Vec<EventWrite<Payload, Meta>> = (1..=10)
        .map(|x| {
            get_event(
                x,
                Payload {
                    name: format!("Todo Event {}", x),
                },
            )
        })
        .collect();
    let mut store = get_store();
    let _ = store
        .append_events(&stream_id, &ExpectedVersion::Any, events)
        .await
        .unwrap();
    let res = store
        .get_events(
            &stream_id,
            &EventsReadRange::FromVersion(EventVersion::new(6)),
        )
        .await
        .unwrap();
    assert_eq!(res.len(), 5);
    are_ascending(res);
}

#[actix_rt::test]
async fn get_events_to_version() {
    let stream_id = get_stream_id();
    let events: Vec<EventWrite<Payload, Meta>> = (1..=10)
        .map(|x| {
            get_event(
                x,
                Payload {
                    name: format!("Todo Event {}", x),
                },
            )
        })
        .collect();
    let mut store = get_store();
    let _ = store
        .append_events(&stream_id, &ExpectedVersion::Any, events)
        .await
        .unwrap();
    let res = store
        .get_events(
            &stream_id,
            &EventsReadRange::ToVersion(EventVersion::new(5)),
        )
        .await
        .unwrap();
    assert_eq!(res.len(), 5);
    are_ascending(res);
}

#[actix_rt::test]
async fn get_events_version_range() {
    let stream_id = get_stream_id();
    let events: Vec<EventWrite<Payload, Meta>> = (1..=10)
        .map(|x| {
            get_event(
                x,
                Payload {
                    name: format!("Todo Event {}", x),
                },
            )
        })
        .collect();
    let mut store = get_store();
    let _ = store
        .append_events(&stream_id, &ExpectedVersion::Any, events)
        .await
        .unwrap();
    let res = store
        .get_events(
            &stream_id,
            &EventsReadRange::VersionRange {
                from_version: EventVersion::new(5),
                to_version: EventVersion::new(7),
            },
        )
        .await
        .unwrap();
    assert_eq!(res.len(), 3);
    are_ascending(res);
}
