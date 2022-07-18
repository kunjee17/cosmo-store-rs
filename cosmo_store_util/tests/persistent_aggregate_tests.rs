#[cfg(test)]
#[macro_use]
extern crate claim;

use cosmo_store::common::i64_event_version::EventVersion;
use cosmo_store::traits::event_store::EventStore;
use cosmo_store::types::event_read_range::EventsReadRange;
use cosmo_store::types::expected_version::ExpectedVersion;
use cosmo_store_sqlx_postgres::event_store_sqlx_postgres::EventStoreSQLXPostgres;
use cosmo_store_tests::event_generator::get_stream_id;
use cosmo_store_util::aggregate::{make_handler, Aggregate};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;
use anyhow::Result;
use futures;
use sqlx::PgPool;
use cosmo_store::types::event_write::EventWrite;

const CONN_BASE: &str = "postgresql://localhost:5432/";

async fn setup(name: &str) {
    println!("Event Store will be initialized here...");
    let conn_str = format!("{}", CONN_BASE);
    let pool = PgPoolOptions::new().connect(&conn_str).await.unwrap();
    let create_db = format!("create database \"{}\" encoding = 'UTF8'", name);
    let _ = sqlx::query(&create_db).execute(&pool).await.unwrap();
    println!("Created {}", name);
}

async fn teardown(name: &str) {
    println!("Event Store will be destroyed here...");
    let conn_str = format!("{}", CONN_BASE);
    let pool = PgPoolOptions::new().connect(&conn_str).await.unwrap();
    let kill_conn = format!(
        "select pg_terminate_backend(pid) from pg_stat_activity where datname='{}'",
        name
    );
    let create_db = format!("drop database if exists \"{}\"", name);
    let _ = sqlx::query(&kill_conn).execute(&pool).await.unwrap();
    let _ = sqlx::query(&create_db).execute(&pool).await.unwrap();
    println!("Destroyed {}", name);
}

async fn get_store<Payload, Meta>(name: &str) -> impl EventStore<Payload, Meta, EventVersion>
where
    Payload: Send + Sync + 'static + Clone + Serialize + for<'de> Deserialize<'de>,
    Meta: Send + Sync + 'static + Clone + Serialize + for<'de> Deserialize<'de>,
{
    let conn_str = format!("{}{}", CONN_BASE, name);
    let pool = PgPoolOptions::new().connect(&conn_str).await.unwrap();
    let store = EventStoreSQLXPostgres::new(&pool, "person").await.unwrap();
    store
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AddTodo {
    pub id: Uuid,
    pub name: String,
    pub is_complete: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RemoveTodo {
    pub id: Uuid,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompleteTodo {
    pub id: Uuid,
}

#[derive(Clone, Debug)]
pub enum TodoCommand {
    AddTodo(AddTodo),
    RemoveTodo(RemoveTodo),
    ClearAllTodo,
    CompleteTodo(CompleteTodo),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TodoEvent {
    TodoAdded(AddTodo),
    TodoRemoved(RemoveTodo),
    AllTodoCleared,
    TodoCompleted(CompleteTodo),
}

impl From<TodoEvent> for EventWrite<TodoEvent, TodoEvent> {
    fn from(t: TodoEvent) -> Self {
        EventWrite {
            id: Default::default(),
            correlation_id: None,
            causation_id: None,
            name: String::from("todo_event"),
            data: t,
            metadata: None,
        }
    }
}

#[derive(Clone, Debug)]
struct Todo {
    id: Uuid,
    name: String,
    is_completed: bool,
}

#[derive(Clone, Debug)]
struct TodoState {
    todos: Vec<Todo>,
}

impl TodoState {
    pub const fn init() -> TodoState {
        TodoState { todos: vec![] }
    }
}

#[derive(Clone, Debug)]
struct TodoAggregate {
    initial_state: TodoState,
}

impl Aggregate<TodoState, TodoCommand, TodoEvent> for TodoAggregate {
    fn init(&self) -> TodoState {
        self.clone().initial_state
    }

    fn apply(&self, state: TodoState, event: &TodoEvent) -> TodoState {
        match event {
            TodoEvent::TodoAdded(t) => {
                let new_state = Todo {
                    id: t.id,
                    name: t.name.clone(),
                    is_completed: t.is_complete,
                };
                TodoState {
                    todos: state
                        .clone()
                        .todos
                        .into_iter()
                        .chain(vec![new_state])
                        .collect(),
                }
            }
            TodoEvent::TodoRemoved(t) => {
                let filtered: Vec<Todo> = state
                    .clone()
                    .todos
                    .into_iter()
                    .filter(|p| p.id != t.id)
                    .collect();
                TodoState { todos: filtered }
            }
            TodoEvent::AllTodoCleared => TodoState { todos: vec![] },
            TodoEvent::TodoCompleted(t) => {
                let find_todo: Todo = state.todos.iter().find(|p| p.id == t.id).unwrap().clone();
                let filtered: Vec<Todo> = state
                    .clone()
                    .todos
                    .into_iter()
                    .filter(|p| p.id != t.id)
                    .collect();
                TodoState {
                    todos: filtered
                        .into_iter()
                        .chain(vec![Todo {
                            is_completed: true,
                            ..find_todo
                        }])
                        .collect(),
                }
            }
        }
    }

    fn execute(&self, state: &TodoState, command: &TodoCommand) -> Result<Vec<TodoEvent>> {
        let res = match command {
            TodoCommand::AddTodo(t) => vec![TodoEvent::TodoAdded(t.clone())],
            TodoCommand::RemoveTodo(t) => vec![TodoEvent::TodoRemoved(t.clone())],
            TodoCommand::ClearAllTodo => vec![TodoEvent::AllTodoCleared],
            TodoCommand::CompleteTodo(t) => vec![TodoEvent::TodoCompleted(t.clone())],
        };

        Ok(res)
    }
}

const TODO_AGGREGATE: TodoAggregate = TodoAggregate {
    initial_state: TodoState::init(),
};

fn get_name() -> String {
    Uuid::new_v4().as_simple().to_string()
}

#[actix_rt::test]
async fn add_state() {
    let name = get_name();
    setup(&name).await;
    let store = get_store(&name).await;
    let stream_id = get_stream_id();

    let res = make_handler(
        &TODO_AGGREGATE,
        &store,
        &TodoCommand::AddTodo(AddTodo {
            id: Default::default(),
            name: "Some Task".to_string(),
            is_complete: false,
        }),
        &stream_id,
        &EventsReadRange::AllEvents,
        &ExpectedVersion::Any,
    )
        .await
        .unwrap();

    assert_eq!(res.len(), 1);

    let state = res
        .iter()
        .fold(TODO_AGGREGATE.init(), |a, b| TODO_AGGREGATE.apply(a, &b.data));

    for r in &res {
        println!("{:?} - {:?}", r.stream_id, r.data);
    }
    assert_eq!(state.todos.len(), 1);

    teardown(&name).await;

}
