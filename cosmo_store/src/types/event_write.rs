use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct EventWrite<Payload, Meta> {
    pub id: Uuid,
    pub correlation_id: Option<String>,
    pub causation_id: Option<String>,
    pub name: String,
    pub data: Payload,
    pub metadata: Option<Meta>,
}
