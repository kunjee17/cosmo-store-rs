use anyhow::Result;
use sqlx::sqlite::SqlitePool;
use sqlx::sqlite::SqliteQueryResult;

#[derive(Debug, Clone)]
pub struct CommandStoreSQLXSqlite {
    pool: SqlitePool,
    table_name: String,
}

impl CommandStoreSQLXSqlite {
    pub fn pool(&self) -> SqlitePool {
        self.pool.clone()
    }
    pub fn table_name(&self) -> String {
        self.table_name.to_string()
    }

    async fn create_command_table(
        pool: &SqlitePool,
        table_name: &str,
    ) -> Result<SqliteQueryResult> {
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
                    (id text primary key , \
                    correlation_id text not null, \
                    causation_id text not null , \
                    data json not null , \
                    name varchar(255) not null , \
                    created_utc date default (datetime('now','utc')))",
            table_name
        );

        let res: SqliteQueryResult = sqlx::query(&command_create_table).execute(pool).await?;
        Ok(res)
    }

    pub async fn new(pool: &SqlitePool, name: &str) -> Result<CommandStoreSQLXSqlite> {
        let command_name = format!("cs_commands_{}", name);
        let _ = CommandStoreSQLXSqlite::create_command_table(pool, &command_name).await?;

        Ok(CommandStoreSQLXSqlite {
            pool: pool.clone(),
            table_name: command_name,
        })
    }
}
