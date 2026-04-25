//! To-do list with file-backed persistence for the `iced-to-do-persist` demo.
//!
//! The on-disk format is one record per line: `<description>:<completed>`,
//! where `completed` is the lowercase string `true` or `false`. Lines with
//! malformed payloads are skipped on load.

use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::Path;

use crate::todo::Todo;

#[derive(Debug, Clone, Default)]
pub struct PersistentTodoList {
    pub new_todo: String,
    pub todos: Vec<Todo>,
}

#[derive(Debug, Clone)]
pub enum Message {
    InputChanged(String),
    AddTodo,
    ToggleTodo(usize),
    DeleteTodo(usize),
    SaveTodos,
    LoadTodos,
}

impl PersistentTodoList {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Mutate state from a [`Message`]. Save / Load operate against
    /// `todo.txt` in the current working directory.
    ///
    /// # Errors
    ///
    /// Forwards I/O errors raised by Save / Load.
    pub fn apply(&mut self, message: Message) -> io::Result<()> {
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
            Message::SaveTodos => self.save(Path::new("todo.txt"))?,
            Message::LoadTodos => self.load(Path::new("todo.txt"))?,
        }
        Ok(())
    }

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

    pub fn toggle(&mut self, index: usize) -> bool {
        if let Some(t) = self.todos.get_mut(index) {
            t.completed = !t.completed;
            true
        } else {
            false
        }
    }

    pub fn delete(&mut self, index: usize) -> bool {
        if index < self.todos.len() {
            self.todos.remove(index);
            true
        } else {
            false
        }
    }

    /// Write the list to `path` as `description:completed` lines.
    ///
    /// # Errors
    ///
    /// Returns [`io::Error`] if the file cannot be opened or written.
    pub fn save(&self, path: &Path) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;
        for todo in &self.todos {
            writeln!(file, "{}:{}", todo.description, todo.completed)?;
        }
        Ok(())
    }

    /// Replace the in-memory list with the contents of `path`. Missing files
    /// reset the list to empty without raising an error (matches the original
    /// demo's `unwrap_or_default()` behavior). Other I/O errors are
    /// propagated.
    ///
    /// # Errors
    ///
    /// Returns [`io::Error`] for I/O errors other than `NotFound` (e.g.
    /// permission denied or the path is a directory).
    pub fn load(&mut self, path: &Path) -> io::Result<()> {
        let contents = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                self.todos.clear();
                return Ok(());
            }
            Err(e) => return Err(e),
        };
        self.todos = contents
            .lines()
            .filter_map(|line| {
                let mut parts = line.splitn(2, ':');
                let description = parts.next()?.to_string();
                let completed = parts.next()?.parse().ok()?;
                Some(Todo {
                    description,
                    completed,
                })
            })
            .collect();
        Ok(())
    }
}

/// Provable contract: save → load round-trips the list byte-for-byte.
///
/// # Errors
///
/// Returns [`contracts::ContractError`] if save or load fails, or if the
/// round-trip changes any todo.
pub fn check_save_load_roundtrip(tmp_path: &Path) -> Result<(), contracts::ContractError> {
    let mut a = PersistentTodoList::new();
    a.todos.push(Todo {
        description: "buy milk".into(),
        completed: false,
    });
    a.todos.push(Todo {
        description: "walk dog".into(),
        completed: true,
    });
    let roundtrip = || -> io::Result<bool> {
        a.save(tmp_path)?;
        let mut b = PersistentTodoList::new();
        b.load(tmp_path)?;
        Ok(a.todos == b.todos)
    };
    let equal = roundtrip().map_err(|e| contracts::ContractError {
        name: "TODO_PERSIST_ROUNDTRIP",
        message: format!("io error: {e}"),
    })?;
    contracts::check(
        "TODO_PERSIST_ROUNDTRIP",
        equal,
        "save → load must preserve todos exactly",
    )
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use std::env;

    fn tmp_path(name: &str) -> std::path::PathBuf {
        env::temp_dir().join(format!("rgfz-{}-{}.txt", name, std::process::id()))
    }

    #[test]
    fn new_is_empty() {
        let t = PersistentTodoList::new();
        assert!(t.todos.is_empty());
    }

    #[test]
    fn input_changed() {
        let mut t = PersistentTodoList::new();
        t.apply(Message::InputChanged("hi".into())).unwrap();
        assert_eq!(t.new_todo, "hi");
    }

    #[test]
    fn add_inserts() {
        let mut t = PersistentTodoList::new();
        t.new_todo = "buy milk".into();
        assert!(t.add());
        assert_eq!(t.todos.len(), 1);
    }

    #[test]
    fn add_rejects_empty() {
        let mut t = PersistentTodoList::new();
        assert!(!t.add());
    }

    #[test]
    fn toggle_flips() {
        let mut t = PersistentTodoList::new();
        t.todos.push(Todo {
            description: "x".into(),
            completed: false,
        });
        assert!(t.toggle(0));
        assert!(t.todos[0].completed);
    }

    #[test]
    fn toggle_out_of_range() {
        let mut t = PersistentTodoList::new();
        assert!(!t.toggle(0));
    }

    #[test]
    fn delete_removes() {
        let mut t = PersistentTodoList::new();
        t.todos.push(Todo {
            description: "a".into(),
            completed: false,
        });
        assert!(t.delete(0));
        assert!(t.todos.is_empty());
    }

    #[test]
    fn delete_out_of_range() {
        let mut t = PersistentTodoList::new();
        assert!(!t.delete(0));
    }

    #[test]
    fn add_via_message() {
        let mut t = PersistentTodoList::new();
        t.new_todo = "x".into();
        t.apply(Message::AddTodo).unwrap();
        assert_eq!(t.todos.len(), 1);
    }

    #[test]
    fn toggle_via_message() {
        let mut t = PersistentTodoList::new();
        t.todos.push(Todo {
            description: "x".into(),
            completed: false,
        });
        t.apply(Message::ToggleTodo(0)).unwrap();
        assert!(t.todos[0].completed);
    }

    #[test]
    fn delete_via_message() {
        let mut t = PersistentTodoList::new();
        t.todos.push(Todo {
            description: "x".into(),
            completed: false,
        });
        t.apply(Message::DeleteTodo(0)).unwrap();
        assert!(t.todos.is_empty());
    }

    #[test]
    fn save_then_load_roundtrips() {
        let path = tmp_path("save_load");
        let mut a = PersistentTodoList::new();
        a.todos.push(Todo {
            description: "buy milk".into(),
            completed: false,
        });
        a.todos.push(Todo {
            description: "walk dog".into(),
            completed: true,
        });
        a.save(&path).unwrap();
        let mut b = PersistentTodoList::new();
        b.load(&path).unwrap();
        assert_eq!(a.todos, b.todos);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn load_missing_file_clears_list() {
        let path = tmp_path("missing").with_extension("nope");
        let _ = std::fs::remove_file(&path);
        let mut t = PersistentTodoList::new();
        t.todos.push(Todo {
            description: "stale".into(),
            completed: false,
        });
        t.load(&path).unwrap();
        assert!(t.todos.is_empty());
    }

    #[test]
    fn load_skips_malformed_lines() {
        let path = tmp_path("malformed");
        std::fs::write(&path, "good:false\nbad-no-colon\n").unwrap();
        let mut t = PersistentTodoList::new();
        t.load(&path).unwrap();
        assert_eq!(t.todos.len(), 1);
        assert_eq!(t.todos[0].description, "good");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn provable_contract_holds() {
        let path = tmp_path("contract");
        check_save_load_roundtrip(&path).expect("contract should hold");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn save_via_message_writes_file() {
        let path = tmp_path("save_msg");
        let prev = env::current_dir().unwrap();
        let dir = path.parent().unwrap().to_path_buf();
        env::set_current_dir(&dir).unwrap();
        // Use a unique cwd to avoid collisions with other tests writing todo.txt.
        let isolated = dir.join(format!("rgfz-cwd-{}", std::process::id()));
        let _ = std::fs::create_dir(&isolated);
        env::set_current_dir(&isolated).unwrap();

        let mut t = PersistentTodoList::new();
        t.todos.push(Todo {
            description: "via msg".into(),
            completed: false,
        });
        t.apply(Message::SaveTodos).unwrap();
        let mut t2 = PersistentTodoList::new();
        t2.apply(Message::LoadTodos).unwrap();
        assert_eq!(t2.todos.len(), 1);
        assert_eq!(t2.todos[0].description, "via msg");

        env::set_current_dir(&prev).unwrap();
        let _ = std::fs::remove_file(isolated.join("todo.txt"));
        let _ = std::fs::remove_dir(&isolated);
    }

    #[test]
    fn load_io_error_is_propagated() {
        // A directory cannot be read as a file (IsADirectory / non-NotFound IO error).
        let dir = env::temp_dir().join(format!("rgfz-isdir-{}", std::process::id()));
        let _ = std::fs::create_dir(&dir);
        let mut t = PersistentTodoList::new();
        let err = t.load(&dir).expect_err("expected IO error on directory");
        assert_ne!(err.kind(), io::ErrorKind::NotFound);
        let _ = std::fs::remove_dir(&dir);
    }

    #[test]
    fn save_io_error_propagates_through_contract() {
        // A path inside a non-existent parent directory cannot be created.
        let bad = env::temp_dir()
            .join(format!("rgfz-no-such-dir-{}", std::process::id()))
            .join("file.txt");
        let err = check_save_load_roundtrip(&bad).expect_err("save should fail");
        assert_eq!(err.name, "TODO_PERSIST_ROUNDTRIP");
        assert!(err.message.starts_with("io error:"));
    }
}
