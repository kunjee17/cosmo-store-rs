use anyhow::Result;
use cosmo_store::traits::event_store::EventStore;
use cosmo_store::types::event_read::EventRead;
use cosmo_store::types::event_read_range::EventsReadRange;
use cosmo_store::types::event_write::EventWrite;
use cosmo_store::types::expected_version::ExpectedVersion;
use serde::{Deserialize, Serialize};

pub trait Aggregate<State, Command, Event> {
    fn init(&self) -> State;
    fn apply(&self, state: State, event: &Event) -> State;
    fn execute(&self, state: &State, command: &Command) -> Result<Vec<Event>>;
}

// Creates a persistent, async command handler for an aggregate given event store.
pub async fn make_handler<State, Command, Event, Meta, Version>(
    aggregate: &impl Aggregate<State, Command, Event>,
    store: &impl EventStore<Event, Meta, Version>,
    command: &Command,
    stream_id: &str,
    range: &EventsReadRange<Version>,
    expected_version: &ExpectedVersion<Version>,
) -> Result<Vec<EventRead<Event, Meta, Version>>>
where
    Version: Eq + PartialEq,
    Event: Into<EventWrite<Event, Meta>> + Clone + Serialize + for<'de> Deserialize<'de>,
    Meta: Clone + Serialize + for<'de> Deserialize<'de>,
{
    let events = store.get_events(stream_id, range).await?;
    let state = events
        .iter()
        .fold(aggregate.init(), |a, b| aggregate.apply(a, &b.data));
    let new_events = aggregate
        .execute(&state, command)?
        .iter()
        .map(|x| x.clone().into())
        .collect();
    store
        .append_events(stream_id, expected_version, new_events)
        .await
}
