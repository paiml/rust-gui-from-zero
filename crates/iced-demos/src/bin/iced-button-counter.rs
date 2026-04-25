//! `iced-button-counter` — single-button incrementing counter.
//!
//! **Provable contract:** `COUNTER_INCREMENT_MONOTONIC` — every Increment
//! raises the count by exactly 1, verified on startup.

use contracts::assert_invariant;
use iced::widget::{button, container, text, Column};
use iced::{Alignment, Element, Sandbox, Settings};
use iced_demos::counter::{check_increment_monotonic, Counter, Message};

struct CounterView {
    state: Counter,
}

impl Sandbox for CounterView {
    type Message = Message;

    fn new() -> Self {
        Self {
            state: Counter::new(),
        }
    }

    fn title(&self) -> String {
        String::from("Button Counter")
    }

    fn update(&mut self, message: Message) {
        let _ = self.state.apply(message);
    }

    fn view(&self) -> Element<'_, Message> {
        let content = Column::new()
            .padding(20)
            .align_items(Alignment::Center)
            .push(button("Increment").on_press(Message::Increment))
            .push(text(format!("Count: {}", self.state.count)).size(24));

        container(content).into()
    }
}

fn main() -> iced::Result {
    assert_invariant!(COUNTER_CONTRACT_HOLDS, check_increment_monotonic().is_ok());
    CounterView::run(Settings::default())
}
