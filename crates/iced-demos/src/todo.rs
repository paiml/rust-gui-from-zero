//! Pure to-do state machine for the `iced-to-do` demo.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Todo {
    pub description: String,
    pub completed: bool,
}

#[derive(Debug, Clone, Default)]
pub struct TodoList {
    pub new_todo: String,
    pub todos: Vec<Todo>,
}

#[derive(Debug, Clone)]
pub enum Message {
    InputChanged(String),
    AddTodo,
    ToggleTodo(usize),
    DeleteTodo(usize),
}

impl TodoList {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Apply a [`Message`] to the list.
    pub fn apply(&mut self, message: Message) {
        match message {
            Message::InputChanged(s) => self.new_todo = s,
            Message::AddTodo => {
                let _ = self.add();
            }
            Message::ToggleTodo(i) => {
                let _ = self.toggle(i);
            }
            Message::DeleteTodo(i) => {
                let _ = self.delete(i);
            }
        }
    }

    /// Push a new todo from the input buffer if it isn't empty after trimming.
    /// Returns whether a new todo was added.
    pub fn add(&mut self) -> bool {
        if self.new_todo.trim().is_empty() {
            return false;
        }
        self.todos.push(Todo {
            description: std::mem::take(&mut self.new_todo),
            completed: false,
        });
        true
    }

    /// Flip the `completed` field on `todos[index]`. No-op for out-of-range.
    pub fn toggle(&mut self, index: usize) -> bool {
        if let Some(t) = self.todos.get_mut(index) {
            t.completed = !t.completed;
            true
        } else {
            false
        }
    }

    /// Remove `todos[index]`. No-op for out-of-range.
    pub fn delete(&mut self, index: usize) -> bool {
        if index < self.todos.len() {
            self.todos.remove(index);
            true
        } else {
            false
        }
    }
}

/// Provable contract: `AddTodo` never inserts an empty-or-whitespace entry.
///
/// # Errors
///
/// Returns [`contracts::ContractError`] if an empty todo gets added.
pub fn check_no_empty_todo() -> Result<(), contracts::ContractError> {
    let mut t = TodoList::new();
    t.new_todo = "   ".into();
    t.apply(Message::AddTodo);
    contracts::check(
        "TODO_NO_EMPTY_INSERT",
        t.todos.is_empty(),
        "AddTodo must reject blank/whitespace input",
    )
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn new_is_empty() {
        let t = TodoList::new();
        assert!(t.todos.is_empty());
        assert!(t.new_todo.is_empty());
    }

    #[test]
    fn input_changed_updates_buffer() {
        let mut t = TodoList::new();
        t.apply(Message::InputChanged("buy milk".into()));
        assert_eq!(t.new_todo, "buy milk");
    }

    #[test]
    fn add_todo_inserts_and_clears_input() {
        let mut t = TodoList::new();
        t.new_todo = "buy milk".into();
        assert!(t.add());
        assert_eq!(t.todos.len(), 1);
        assert_eq!(t.todos[0].description, "buy milk");
        assert!(!t.todos[0].completed);
        assert!(t.new_todo.is_empty());
    }

    #[test]
    fn add_todo_rejects_empty() {
        let mut t = TodoList::new();
        assert!(!t.add());
        assert!(t.todos.is_empty());
    }

    #[test]
    fn add_todo_rejects_whitespace() {
        let mut t = TodoList::new();
        t.new_todo = "   \t  ".into();
        assert!(!t.add());
        assert!(t.todos.is_empty());
    }

    #[test]
    fn add_via_message() {
        let mut t = TodoList::new();
        t.apply(Message::InputChanged("walk dog".into()));
        t.apply(Message::AddTodo);
        assert_eq!(t.todos.len(), 1);
    }

    #[test]
    fn toggle_flips_completed() {
        let mut t = TodoList::new();
        t.new_todo = "x".into();
        t.add();
        assert!(t.toggle(0));
        assert!(t.todos[0].completed);
        assert!(t.toggle(0));
        assert!(!t.todos[0].completed);
    }

    #[test]
    fn toggle_via_message() {
        let mut t = TodoList::new();
        t.new_todo = "x".into();
        t.add();
        t.apply(Message::ToggleTodo(0));
        assert!(t.todos[0].completed);
    }

    #[test]
    fn toggle_out_of_range_is_noop() {
        let mut t = TodoList::new();
        assert!(!t.toggle(99));
    }

    #[test]
    fn delete_removes_at_index() {
        let mut t = TodoList::new();
        t.new_todo = "a".into();
        t.add();
        t.new_todo = "b".into();
        t.add();
        assert!(t.delete(0));
        assert_eq!(t.todos.len(), 1);
        assert_eq!(t.todos[0].description, "b");
    }

    #[test]
    fn delete_via_message() {
        let mut t = TodoList::new();
        t.new_todo = "a".into();
        t.add();
        t.apply(Message::DeleteTodo(0));
        assert!(t.todos.is_empty());
    }

    #[test]
    fn delete_out_of_range_is_noop() {
        let mut t = TodoList::new();
        assert!(!t.delete(0));
    }

    #[test]
    fn provable_contract_holds() {
        check_no_empty_todo().expect("contract should hold");
    }
}
