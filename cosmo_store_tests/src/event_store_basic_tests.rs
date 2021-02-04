use crate::event_generator::{get_event, get_stream_id};
use cosmo_store::traits::event_store::EventStore;
use cosmo_store::types::event_read::EventRead;
use cosmo_store::types::event_read_range::EventsReadRange;
use cosmo_store::types::event_write::EventWrite;
use cosmo_store::types::expected_version::ExpectedVersion;
use serde::{Deserialize, Serialize};
use std::ops::RangeInclusive;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Payload {
    pub name: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Meta {}

fn get_events(x: RangeInclusive<i32>) -> Vec<EventWrite<Payload, Meta>> {
    x.map(|x| {
        get_event(
            x,
            Payload {
                name: format!("Todo Event {}", x),
            },
        )
    })
    .collect()
}

pub fn check_position<V, G>(greater_than: G, return_val: V) -> V
where
    G: FnOnce() -> bool,
{
    assert_eq!(greater_than(), true);
    return_val
}

pub async fn append_event<V, F>(mut store: Box<dyn EventStore<Payload, Meta, V>>, assert: F)
where
    F: FnOnce(EventRead<Payload, Meta, V>),
{
    let stream_id = get_stream_id();
    let event = get_event(
        0,
        Payload {
            name: "Todo Event".to_string(),
        },
    );
    let res = store
        .append_event(&stream_id, &ExpectedVersion::Any, &event)
        .await
        .unwrap();
    assert(res);
}

pub async fn append_100_events<V, F>(mut store: Box<dyn EventStore<Payload, Meta, V>>, assert: F)
where
    F: FnOnce(Vec<EventRead<Payload, Meta, V>>),
{
    let stream_id = get_stream_id();
    let events: Vec<EventWrite<Payload, Meta>> = get_events(1..=100);
    let res: Vec<EventRead<Payload, Meta, V>> = store
        .append_events(&stream_id, &ExpectedVersion::Any, events)
        .await
        .unwrap();

    assert(res);
}

pub async fn get_single_event<V, F>(
    mut store: Box<dyn EventStore<Payload, Meta, V>>,
    version: &V,
    assert: F,
) where
    F: FnOnce(EventRead<Payload, Meta, V>),
{
    let stream_id = get_stream_id();
    let events: Vec<EventWrite<Payload, Meta>> = get_events(1..=10);
    let _ = store
        .append_events(&stream_id, &ExpectedVersion::Any, events)
        .await
        .unwrap();

    let res = store.get_event(&stream_id, version).await.unwrap();
    assert(res);
}

pub async fn get_all_events<V, F>(mut store: Box<dyn EventStore<Payload, Meta, V>>, assert: F)
where
    F: FnOnce(Vec<EventRead<Payload, Meta, V>>),
{
    let stream_id = get_stream_id();
    let events: Vec<EventWrite<Payload, Meta>> = get_events(1..=10);
    let _ = store
        .append_events(&stream_id, &ExpectedVersion::Any, events)
        .await
        .unwrap();
    let res = store
        .get_events(&stream_id, &EventsReadRange::AllEvents)
        .await
        .unwrap();
    assert(res);
}

pub async fn get_events_from_version<V, F>(
    mut store: Box<dyn EventStore<Payload, Meta, V>>,
    from_version: V,
    assert: F,
) where
    F: FnOnce(Vec<EventRead<Payload, Meta, V>>),
{
    let stream_id = get_stream_id();
    let events: Vec<EventWrite<Payload, Meta>> = get_events(1..=10);
    let _ = store
        .append_events(&stream_id, &ExpectedVersion::Any, events)
        .await
        .unwrap();
    let res = store
        .get_events(&stream_id, &EventsReadRange::FromVersion(from_version))
        .await
        .unwrap();
    assert(res);
}

pub async fn get_events_to_version<V, F>(
    mut store: Box<dyn EventStore<Payload, Meta, V>>,
    to_version: V,
    assert: F,
) where
    F: FnOnce(Vec<EventRead<Payload, Meta, V>>),
{
    let stream_id = get_stream_id();
    let events: Vec<EventWrite<Payload, Meta>> = get_events(1..=10);
    let _ = store
        .append_events(&stream_id, &ExpectedVersion::Any, events)
        .await
        .unwrap();
    let res = store
        .get_events(&stream_id, &EventsReadRange::ToVersion(to_version))
        .await
        .unwrap();
    assert(res);
}

pub async fn get_events_version_range<V, F>(
    mut store: Box<dyn EventStore<Payload, Meta, V>>,
    from_version: V,
    to_version: V,
    assert: F,
) where
    F: FnOnce(Vec<EventRead<Payload, Meta, V>>),
{
    let stream_id = get_stream_id();
    let events: Vec<EventWrite<Payload, Meta>> = get_events(1..=10);
    let _ = store
        .append_events(&stream_id, &ExpectedVersion::Any, events)
        .await
        .unwrap();
    let res = store
        .get_events(
            &stream_id,
            &EventsReadRange::VersionRange {
                from_version,
                to_version,
            },
        )
        .await
        .unwrap();
    assert(res);
}
