use crate::event_generator::{get_event, get_events, get_stream_id};
use anyhow::Result;
use cosmo_store::traits::event_store::EventStore;
use cosmo_store::types::event_read::EventRead;
use cosmo_store::types::event_read_range::EventsReadRange;
use cosmo_store::types::event_stream::EventStream;
use cosmo_store::types::event_write::EventWrite;
use cosmo_store::types::expected_version::ExpectedVersion;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Payload {
    pub name: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Meta {}

pub fn check_position<V, G>(greater_than: G, return_val: V) -> V
where
    G: FnOnce() -> bool,
    V: Debug + Eq + PartialEq,
{
    assert_eq!(greater_than(), true);
    return_val
}

pub async fn append_event<V, F>(mut store: Box<dyn EventStore<Payload, Meta, V>>, assert: F)
where
    F: FnOnce(EventRead<Payload, Meta, V>),
    V: Debug + Eq + PartialEq,
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
    V: Debug + Eq + PartialEq,
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
    V: Debug + Eq + PartialEq,
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
    V: Debug + Eq + PartialEq,
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
    V: Debug + Eq + PartialEq,
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
    V: Debug + Eq + PartialEq,
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
    V: Debug + Eq + PartialEq,
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

pub async fn fails_to_append_to_existing_version<V, F>(
    mut store: Box<dyn EventStore<Payload, Meta, V>>,
    version: V,
    assert: F,
) where
    F: FnOnce(Result<EventRead<Payload, Meta, V>>),
    V: Debug + Eq + PartialEq,
{
    let stream_id = get_stream_id();

    let _ = store
        .append_event(
            &stream_id,
            &ExpectedVersion::Any,
            &get_event(
                1,
                Payload {
                    name: "Todo Event".to_string(),
                },
            ),
        )
        .await
        .unwrap();

    let res = store
        .append_event(
            &stream_id,
            &ExpectedVersion::Exact(version),
            &get_event(
                1,
                Payload {
                    name: "Todo Event".to_string(),
                },
            ),
        )
        .await;

    assert(res);
}

pub async fn fails_to_append_to_existing_stream_if_is_not_expected_to_exist<V, F>(
    mut store: Box<dyn EventStore<Payload, Meta, V>>,
    assert: F,
) where
    F: FnOnce(Result<EventRead<Payload, Meta, V>>),
    V: Debug + Eq + PartialEq,
{
    let stream_id = get_stream_id();

    let _ = store
        .append_event(
            &stream_id,
            &ExpectedVersion::Any,
            &get_event(
                1,
                Payload {
                    name: "Todo Event".to_string(),
                },
            ),
        )
        .await
        .unwrap();

    let res = store
        .append_event(
            &stream_id,
            &ExpectedVersion::NoStream,
            &get_event(
                1,
                Payload {
                    name: "Todo Event".to_string(),
                },
            ),
        )
        .await;

    assert(res);
}

pub async fn appending_no_events_does_not_affect_stream_metadata<V, F>(
    mut store: Box<dyn EventStore<Payload, Meta, V>>,
    version: &ExpectedVersion<V>,
    assert: F,
) where
    F: FnOnce(EventStream<V>, EventStream<V>),
    V: Debug + Eq + PartialEq,
{
    let stream_id = get_stream_id();

    let _ = store
        .append_event(
            &stream_id,
            version,
            &get_event(
                1,
                Payload {
                    name: "Todo Event".to_string(),
                },
            ),
        )
        .await
        .unwrap();

    let stream = store.get_stream(&stream_id).await.unwrap();

    let _ = store
        .append_events(&stream_id, &ExpectedVersion::Any, Vec::new())
        .await
        .unwrap();

    let stream_after_append = store.get_stream(&stream_id).await.unwrap();

    assert(stream, stream_after_append)
}

pub async fn appending_1000_events_can_be_read_back<V, F>(
    mut store: Box<dyn EventStore<Payload, Meta, V>>,
    assert: F,
) where
    F: FnOnce(EventStream<V>, Vec<EventRead<Payload, Meta, V>>),
    V: Debug + Eq + PartialEq,
{
    let stream_id = get_stream_id();
    for i in 0..10 {
        let events = get_events(i * 100..=(i * 100) + 99);
        let _ = store
            .append_events(&stream_id, &ExpectedVersion::Any, events)
            .await
            .unwrap();
    }

    let stream = store.get_stream(&stream_id).await.unwrap();
    let events = store
        .get_events(&stream_id, &EventsReadRange::AllEvents)
        .await
        .unwrap();

    assert(stream, events)
}

pub async fn can_read_events_by_correlation_id<V, F>(
    mut store: Box<dyn EventStore<Payload, Meta, V>>,
    assert: F,
) where
    F: FnOnce(Vec<EventRead<Payload, Meta, V>>),
    V: Debug + Eq + PartialEq,
{
    async fn add_events_to_stream<V: Eq + PartialEq>(
        store: &mut Box<dyn EventStore<Payload, Meta, V>>,
        corr_id: Uuid,
        i: i32,
    ) {
        let events = get_events(1..=10)
            .iter()
            .map(|x| EventWrite {
                correlation_id: Some(corr_id),
                ..x.clone()
            })
            .collect();
        let stream_id = format!("CORR_{0}", i);
        store
            .append_events(&stream_id, &ExpectedVersion::Any, events)
            .await
            .unwrap();
    }

    let corr_id = Uuid::new_v4();

    for i in 1..=3 {
        add_events_to_stream(&mut store, corr_id, i).await;
    }

    let different_corr_id = Uuid::new_v4();
    for i in 1..=3 {
        add_events_to_stream(&mut store, different_corr_id, i).await;
    }

    let events = store.get_events_by_correlation_id(&corr_id).await.unwrap();

    assert(events)
}
