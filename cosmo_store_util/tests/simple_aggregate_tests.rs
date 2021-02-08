use anyhow::Result;
use cosmo_store::types::event_write::EventWrite;
use cosmo_store_util::aggregate::Aggregate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

#[test]
fn it_works() {
    assert_eq!(2 + 2, 4);
}

#[test]
fn init_state() {
    assert_eq!(
        TODO_AGGREGATE.init().todos.len(),
        TodoState::init().todos.len()
    )
}

#[test]
fn add_state() {
    let events = TODO_AGGREGATE
        .execute(
            &TODO_AGGREGATE.init(),
            &TodoCommand::AddTodo(AddTodo {
                id: Default::default(),
                name: "Some Task".to_string(),
                is_complete: false,
            }),
        )
        .unwrap();

    assert_eq!(events.len(), 1);
    let state = events
        .iter()
        .fold(TODO_AGGREGATE.init(), |a, b| TODO_AGGREGATE.apply(a, &b));

    assert_eq!(state.todos.len(), 1);
}
