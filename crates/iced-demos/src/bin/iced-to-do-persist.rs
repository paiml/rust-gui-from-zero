//! `iced-to-do-persist` — to-do list with file-backed persistence.
//!
//! **Provable contract:** `TODO_PERSIST_ROUNDTRIP` — `save → load` preserves
//! the list byte-for-byte, verified on startup against a temp file.

use contracts::assert_invariant;
use iced::widget::{button, checkbox, column, row, text_input};
use iced::{Alignment, Element, Sandbox, Settings};
use iced_demos::todo_persist::{check_save_load_roundtrip, Message, PersistentTodoList};

struct PersistView {
    state: PersistentTodoList,
}

impl Sandbox for PersistView {
    type Message = Message;

    fn new() -> Self {
        Self {
            state: PersistentTodoList::new(),
        }
    }

    fn title(&self) -> String {
        String::from("Iced Todo List (Persistent)")
    }

    fn update(&mut self, message: Message) {
        let _ = self.state.apply(message);
    }

    fn view(&self) -> Element<'_, Message> {
        let input = text_input("New todo...", &self.state.new_todo)
            .on_input(Message::InputChanged)
            .on_submit(Message::AddTodo);
        let add_button = button("Add").on_press(Message::AddTodo);
        let save_button = button("Save").on_press(Message::SaveTodos);
        let load_button = button("Load").on_press(Message::LoadTodos);

        let todos =
            self.state
                .todos
                .iter()
                .enumerate()
                .fold(column![].spacing(10), |col, (i, todo)| {
                    col.push(
                        row![
                            checkbox(&todo.description, todo.completed, move |_| {
                                Message::ToggleTodo(i)
                            }),
                            button("Delete").on_press(Message::DeleteTodo(i))
                        ]
                        .spacing(20)
                        .align_items(Alignment::Center),
                    )
                });

        column![
            row![input, add_button].spacing(10),
            row![save_button, load_button].spacing(10),
            todos,
        ]
        .padding(20)
        .spacing(20)
        .into()
    }
}

fn main() -> iced::Result {
    let tmp = std::env::temp_dir().join(format!("rgfz-startup-{}.txt", std::process::id()));
    assert_invariant!(
        TODO_PERSIST_CONTRACT_HOLDS,
        check_save_load_roundtrip(&tmp).is_ok()
    );
    let _ = std::fs::remove_file(&tmp);
    PersistView::run(Settings::default())
}
