use chrono::{DateTime, Utc};
use uuid::Uuid;

/**
Let’s say every message has 3 ids. 1 is its id.
Another is correlation the last it causation.
If you are responding to a message,
you copy its correlation id as your correlation id,
its message id is your causation id.
This allows you to see an entire conversation (correlation id) or
to see what causes what (causation id).
*/
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandWrite<Payload> {
    pub id: Uuid,
    pub correlation_id: Uuid,
    pub causation_id: Uuid,
    pub data: Payload,
    pub name: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandRead<Payload> {
    pub id: Uuid,
    pub correlation_id: Uuid,
    pub causation_id: Uuid,
    pub data: Payload,
    pub name: String,
    pub created_utc: DateTime<Utc>,
}
