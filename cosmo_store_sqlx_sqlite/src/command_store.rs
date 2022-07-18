use crate::command_store_sqlx_sqlite::CommandStoreSQLXSqlite;
use anyhow::Result;
use async_trait::async_trait;
use cosmo_store::traits::command_store::CommandStore;
use cosmo_store::types::command_write::CommandWrite;
use serde::{Deserialize, Serialize};

#[async_trait]
impl<Payload> CommandStore<Payload> for CommandStoreSQLXSqlite
where
    Payload: Send + Sync + 'static + Clone + Serialize + for<'de> Deserialize<'de>,
{
    async fn append_command(&self, payload: &CommandWrite<Payload>) -> Result<()> {
        //     insert into cs_command_person (id, correlation_id, causation_id, data, name)
        //     values ('ed56bdfd-8fb2-4c91-aea4-72a74c986985', 'ed56bdfd-8fb2-4c91-aea4-72a74c986985', 'ed56bdfd-8fb2-4c91-aea4-72a74c986985', '{
        //     "name": "kunjan j dalal"
        // }', 'do something');
        let insert_command = format!(
            "insert into {0} \
        (id, correlation_id, causation_id, data, name) \
        values ($1, $2, $3, $4, $5)",
            self.table_name()
        );
        let data = serde_json::to_value(payload.data.clone())?;
        let mut tr = self.pool().begin().await?;
        let _ = sqlx::query(&insert_command)
            .bind(payload.id)
            .bind(payload.correlation_id)
            .bind(payload.causation_id)
            .bind(data)
            .bind(payload.name.clone())
            .execute(&mut tr)
            .await?;

        tr.commit().await?;
        Ok(())
    }
}
