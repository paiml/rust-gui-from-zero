//! `iced-hello` — minimal "Hello, Iced!" window.
//!
//! **Provable contract:** `HELLO_GREETING_NON_EMPTY` — title and body strings
//! are non-empty (a blank window would fail QA).

use contracts::assert_invariant;
use iced::widget::{container, text};
use iced::{Element, Sandbox, Settings};
use iced_demos::hello::{check_greeting_non_empty, BODY, TITLE};

struct HelloView;

impl Sandbox for HelloView {
    type Message = ();

    fn new() -> Self {
        Self
    }

    fn title(&self) -> String {
        TITLE.to_string()
    }

    fn update(&mut self, (): ()) {}

    fn view(&self) -> Element<'_, ()> {
        container(text(BODY)).into()
    }
}

fn main() -> iced::Result {
    assert_invariant!(HELLO_CONTRACT_HOLDS, check_greeting_non_empty().is_ok());
    HelloView::run(Settings::default())
}
