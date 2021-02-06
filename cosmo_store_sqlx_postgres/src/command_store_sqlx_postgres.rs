use anyhow::Result;
use sqlx::postgres::PgDone;
use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct CommandStoreSQLXPostgres {
    pool: PgPool,
    table_name: String,
}

impl CommandStoreSQLXPostgres {
    pub fn pool(&self) -> PgPool {
        self.pool.clone()
    }
    pub fn table_name(&self) -> String {
        self.table_name.to_string()
    }

    async fn create_command_table(pool: &PgPool, table_name: &str) -> Result<PgDone> {
        // create table if not exists cs_command_person (
        //     id uuid primary key ,
        // correlation_id uuid not null ,
        // causation_id uuid not null ,
        // data jsonb not null ,
        // name varchar(255) not null ,
        // created_utc timestamptz default current_timestamp
        // )

        let command_create_table = format!(
            "create table if not exists {0} \
                    (id uuid primary key , \
                    correlation_id uuid not null, \
                    causation_id uuid not null , \
                    data jsonb not null , \
                    name varchar(255) not null , \
                    created_utc timestamptz default current_timestamp)",
            table_name
        );

        let res = sqlx::query(&command_create_table).execute(pool).await?;
        Ok(res)
    }

    pub async fn new(pool: &PgPool, name: &str) -> Result<CommandStoreSQLXPostgres> {
        let command_name = format!("cs_commands_{}", name);
        let _ = CommandStoreSQLXPostgres::create_command_table(pool, &command_name).await?;

        Ok(CommandStoreSQLXPostgres {
            pool: pool.clone(),
            table_name: command_name,
        })
    }
}
