use crate::types::command_write::CommandWrite;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait CommandStore<Payload> {
    async fn append_command(&self, payload: &CommandWrite<Payload>) -> Result<()>;
}
