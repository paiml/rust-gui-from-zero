//! `iced-calc` — desktop calculator.
//!
//! **Provable contract:** `CALC_CALCULATE_REQUIRES_OPERATION` — pressing `=`
//! before any operator must leave the display unchanged. The state machine
//! that implements this is in `iced_demos::calculator`; the contract is
//! re-verified on startup so a regression panics before the window opens.

use contracts::assert_invariant;
use iced::widget::{button, container, text, text_input, Column, Row};
use iced::{Alignment, Color, Element, Length, Sandbox, Settings};
use iced_demos::calculator::{
    check_calculate_no_op_when_no_operation, Calculator, Message, Operation,
};

struct CalcView {
    state: Calculator,
}

impl Sandbox for CalcView {
    type Message = Message;

    fn new() -> Self {
        Self {
            state: Calculator::new(),
        }
    }

    fn title(&self) -> String {
        String::from("Simple Calculator")
    }

    fn update(&mut self, message: Message) {
        let _ = self.state.apply(message);
    }

    fn view(&self) -> Element<'_, Message> {
        let subtotal = container(
            text(format!("Subtotal: {}", self.state.display))
                .size(24)
                .style(Color::from([0.5, 0.5, 0.5])),
        )
        .padding(10);

        let input = text_input("Enter a number", &self.state.current_value)
            .on_input(Message::InputNumber)
            .padding(10)
            .size(20)
            .width(Length::Fixed(150.0));

        let buttons = Row::new()
            .spacing(10)
            .push(button("+").on_press(Message::Operation(Operation::Add)))
            .push(button("-").on_press(Message::Operation(Operation::Subtract)))
            .push(button("*").on_press(Message::Operation(Operation::Multiply)))
            .push(button("/").on_press(Message::Operation(Operation::Divide)));

        let controls = Row::new()
            .spacing(10)
            .push(button("Calculate").on_press(Message::Calculate))
            .push(button("Clear").on_press(Message::Clear));

        let content = Column::new()
            .padding(20)
            .spacing(20)
            .align_items(Alignment::Center)
            .push(subtotal)
            .push(input)
            .push(buttons)
            .push(controls);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

fn main() -> iced::Result {
    assert_invariant!(
        CALC_CONTRACT_HOLDS,
        check_calculate_no_op_when_no_operation().is_ok()
    );
    CalcView::run(Settings::default())
}
