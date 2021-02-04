use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct EventWrite<Payload, Meta> {
    pub id: Uuid,
    pub correlation_id: Option<Uuid>,
    pub causation_id: Option<Uuid>,
    pub name: String,
    pub data: Payload,
    pub metadata: Option<Meta>,
}
