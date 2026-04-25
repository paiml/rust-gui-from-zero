//! `iced-to-do` — in-memory to-do list.
//!
//! **Provable contract:** `TODO_NO_EMPTY_INSERT` — `AddTodo` rejects
//! blank/whitespace input, verified on startup.

use contracts::assert_invariant;
use iced::widget::{button, checkbox, column, row, text_input};
use iced::{Alignment, Element, Sandbox, Settings};
use iced_demos::todo::{check_no_empty_todo, Message, TodoList};

struct TodoView {
    state: TodoList,
}

impl Sandbox for TodoView {
    type Message = Message;

    fn new() -> Self {
        Self {
            state: TodoList::new(),
        }
    }

    fn title(&self) -> String {
        String::from("Iced Todo List")
    }

    fn update(&mut self, message: Message) {
        self.state.apply(message);
    }

    fn view(&self) -> Element<'_, Message> {
        let input = text_input("New todo...", &self.state.new_todo)
            .on_input(Message::InputChanged)
            .on_submit(Message::AddTodo);
        let add_button = button("Add").on_press(Message::AddTodo);
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
        column![row![input, add_button].spacing(10), todos]
            .padding(20)
            .spacing(20)
            .into()
    }
}

fn main() -> iced::Result {
    assert_invariant!(TODO_CONTRACT_HOLDS, check_no_empty_todo().is_ok());
    TodoView::run(Settings::default())
}
