use uuid::Uuid;
use cosmo_store::types::event_write::EventWrite;

pub fn get_event<Payload, Meta>(i: i32, data: Payload) -> EventWrite<Payload, Meta> {
    let (corr, caus) = match (i % 2, i % 3) {
        (0, _) => (Some(Uuid::new_v4().to_string()), None),
        (_, 0) => (None, Some(Uuid::new_v4().to_string())),
        (_, _) => (None, None),
    };

    EventWrite {
        id: Uuid::new_v4(),
        correlation_id: corr,
        causation_id: caus,
        name: format!("Created_{}", i),
        data,
        metadata: None,
    }
}

pub fn get_stream_id() -> String {
    format!("TestStream_{}", Uuid::new_v4().to_string())
}
