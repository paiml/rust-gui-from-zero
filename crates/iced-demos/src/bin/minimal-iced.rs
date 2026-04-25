//! `minimal-iced` — +/-/clear counter.
//!
//! **Provable contract:** `MINIMAL_CLEAR_RESETS` — Clear always zeroes the
//! value regardless of prior state.

use contracts::assert_invariant;
use iced::widget::{button, column, text};
use iced::{Element, Sandbox, Settings};
use iced_demos::minimal::{check_clear_resets_to_zero, Message, MinimalApp};

struct MinimalView {
    state: MinimalApp,
}

impl Sandbox for MinimalView {
    type Message = Message;

    fn new() -> Self {
        Self {
            state: MinimalApp::new(),
        }
    }

    fn title(&self) -> String {
        String::from("Minimal Iced App")
    }

    fn update(&mut self, message: Message) {
        let _ = self.state.apply(message);
    }

    fn view(&self) -> Element<'_, Message> {
        column![
            text(self.state.value.to_string()).size(50),
            button("+").on_press(Message::Increment),
            button("-").on_press(Message::Decrement),
            button("Clear").on_press(Message::Clear),
        ]
        .padding(20)
        .into()
    }
}

fn main() -> iced::Result {
    assert_invariant!(MINIMAL_CONTRACT_HOLDS, check_clear_resets_to_zero().is_ok());
    MinimalView::run(Settings::default())
}
