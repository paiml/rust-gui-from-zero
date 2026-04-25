//! Pure calculator state machine for the `iced-calc` demo.
//!
//! The view layer in `src/bin/iced-calc.rs` calls [`Calculator::apply`] in
//! response to button presses; this module owns no widgets and never touches
//! iced — making the entire arithmetic pipeline trivially unit-testable.

use std::fmt;

/// Binary arithmetic operations the calculator supports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Add => "+",
            Self::Subtract => "-",
            Self::Multiply => "*",
            Self::Divide => "/",
        })
    }
}

/// Messages emitted by the iced view; consumed by [`Calculator::apply`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    InputNumber(String),
    Operation(Operation),
    Calculate,
    Clear,
}

/// Calculator state. The [`apply`] method is the only mutator.
#[derive(Debug, Clone)]
pub struct Calculator {
    pub current_value: String,
    pub previous_value: Option<f64>,
    pub operation: Option<Operation>,
    pub display: String,
}

impl Default for Calculator {
    fn default() -> Self {
        Self::new()
    }
}

impl Calculator {
    /// Construct a calculator showing `0`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            current_value: String::new(),
            previous_value: None,
            operation: None,
            display: String::from("0"),
        }
    }

    /// Apply a [`Message`] and return the resulting display string.
    pub fn apply(&mut self, message: Message) -> &str {
        match message {
            Message::InputNumber(num) => self.input(num),
            Message::Operation(op) => self.operation(op),
            Message::Calculate => self.calculate(),
            Message::Clear => self.clear(),
        }
        &self.display
    }

    fn input(&mut self, num: String) {
        self.current_value = num;
        if self.operation.is_none() {
            self.display.clone_from(&self.current_value);
        }
    }

    fn operation(&mut self, op: Operation) {
        let Ok(value) = self.current_value.parse::<f64>() else {
            return;
        };
        if let Some(prev) = self.previous_value {
            if let Some(result) = self.eval(prev, value) {
                self.display = format_number(result);
                self.previous_value = Some(result);
            }
        } else {
            self.previous_value = Some(value);
            self.display = format_number(value);
        }
        self.operation = Some(op);
        self.current_value.clear();
    }

    fn calculate(&mut self) {
        if let (Some(prev), Some(_op), Ok(current)) = (
            self.previous_value,
            self.operation,
            self.current_value.parse::<f64>(),
        ) {
            if let Some(result) = self.eval(prev, current) {
                let formatted = format_number(result);
                self.display.clone_from(&formatted);
                self.current_value.clone_from(&formatted);
                self.previous_value = None;
                self.operation = None;
            }
        }
    }

    fn clear(&mut self) {
        self.current_value.clear();
        self.previous_value = None;
        self.operation = None;
        self.display = String::from("0");
    }

    /// Evaluate `a <op> b` using the currently-pending operation.
    /// Returns `None` if no operation is set or division-by-zero is requested.
    fn eval(&self, a: f64, b: f64) -> Option<f64> {
        match self.operation? {
            Operation::Add => Some(a + b),
            Operation::Subtract => Some(a - b),
            Operation::Multiply => Some(a * b),
            Operation::Divide => {
                if b == 0.0 {
                    None
                } else {
                    Some(a / b)
                }
            }
        }
    }
}

fn format_number(n: f64) -> String {
    if (n.fract()).abs() < f64::EPSILON {
        #[allow(clippy::cast_possible_truncation)]
        let truncated = n as i64;
        format!("{truncated}")
    } else {
        format!("{n}")
    }
}

/// Provable contract: `apply(Calculate)` is a no-op when no operation is
/// pending. The view depends on this so that pressing `=` before any operator
/// keeps the screen on the last input rather than silently zeroing.
///
/// # Errors
///
/// Returns [`contracts::ContractError`] if the invariant is violated.
pub fn check_calculate_no_op_when_no_operation() -> Result<(), contracts::ContractError> {
    let mut calc = Calculator::new();
    calc.apply(Message::InputNumber(String::from("42")));
    let display = calc.display.clone();
    calc.apply(Message::Calculate);
    contracts::check(
        "CALC_CALCULATE_REQUIRES_OPERATION",
        calc.display == display,
        "Calculate without a pending operation must not alter the display",
    )
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    fn calc() -> Calculator {
        Calculator::new()
    }

    fn run(c: &mut Calculator, msgs: &[Message]) -> String {
        let mut last = c.display.clone();
        for m in msgs {
            last = c.apply(m.clone()).to_string();
        }
        last
    }

    #[test]
    fn new_displays_zero() {
        assert_eq!(calc().display, "0");
    }

    #[test]
    fn default_matches_new() {
        let a: Calculator = Calculator::default();
        let b: Calculator = Calculator::new();
        assert_eq!(a.display, b.display);
    }

    #[test]
    fn input_updates_current_and_display() {
        let mut c = calc();
        let r = run(&mut c, &[Message::InputNumber("123".into())]);
        assert_eq!(r, "123");
        assert_eq!(c.current_value, "123");
    }

    #[test]
    fn input_then_operation_buffers_first_operand() {
        let mut c = calc();
        run(
            &mut c,
            &[
                Message::InputNumber("5".into()),
                Message::Operation(Operation::Add),
            ],
        );
        assert_eq!(c.previous_value, Some(5.0));
        assert_eq!(c.operation, Some(Operation::Add));
        assert!(c.current_value.is_empty());
    }

    #[test]
    fn add() {
        let mut c = calc();
        let r = run(
            &mut c,
            &[
                Message::InputNumber("5".into()),
                Message::Operation(Operation::Add),
                Message::InputNumber("3".into()),
                Message::Calculate,
            ],
        );
        assert_eq!(r, "8");
        assert_eq!(c.previous_value, None);
        assert_eq!(c.operation, None);
    }

    #[test]
    fn subtract() {
        let mut c = calc();
        let r = run(
            &mut c,
            &[
                Message::InputNumber("10".into()),
                Message::Operation(Operation::Subtract),
                Message::InputNumber("3".into()),
                Message::Calculate,
            ],
        );
        assert_eq!(r, "7");
    }

    #[test]
    fn multiply() {
        let mut c = calc();
        let r = run(
            &mut c,
            &[
                Message::InputNumber("4".into()),
                Message::Operation(Operation::Multiply),
                Message::InputNumber("3".into()),
                Message::Calculate,
            ],
        );
        assert_eq!(r, "12");
    }

    #[test]
    fn divide() {
        let mut c = calc();
        let r = run(
            &mut c,
            &[
                Message::InputNumber("10".into()),
                Message::Operation(Operation::Divide),
                Message::InputNumber("2".into()),
                Message::Calculate,
            ],
        );
        assert_eq!(r, "5");
    }

    #[test]
    fn divide_by_zero_holds_state() {
        let mut c = calc();
        run(
            &mut c,
            &[
                Message::InputNumber("10".into()),
                Message::Operation(Operation::Divide),
                Message::InputNumber("0".into()),
                Message::Calculate,
            ],
        );
        // eval returned None → calculate is a no-op; previous_value stays Some(10.0)
        assert_eq!(c.previous_value, Some(10.0));
        assert_eq!(c.operation, Some(Operation::Divide));
    }

    #[test]
    fn chained_operations_use_running_total() {
        let mut c = calc();
        let r = run(
            &mut c,
            &[
                Message::InputNumber("2".into()),
                Message::Operation(Operation::Add),
                Message::InputNumber("3".into()),
                Message::Operation(Operation::Multiply),
                Message::InputNumber("4".into()),
                Message::Calculate,
            ],
        );
        // (2+3) then * 4 = 20
        assert_eq!(r, "20");
    }

    #[test]
    fn operation_with_unparseable_current_is_noop() {
        let mut c = calc();
        c.current_value = "xyz".into();
        let prev_state = c.clone();
        c.apply(Message::Operation(Operation::Add));
        assert_eq!(c.display, prev_state.display);
        assert_eq!(c.previous_value, prev_state.previous_value);
    }

    #[test]
    fn calculate_without_operation_is_noop() {
        let mut c = calc();
        run(&mut c, &[Message::InputNumber("42".into())]);
        let before = c.display.clone();
        c.apply(Message::Calculate);
        assert_eq!(c.display, before);
    }

    #[test]
    fn clear_resets_all_state() {
        let mut c = calc();
        run(
            &mut c,
            &[
                Message::InputNumber("5".into()),
                Message::Operation(Operation::Add),
                Message::InputNumber("3".into()),
                Message::Clear,
            ],
        );
        assert_eq!(c.display, "0");
        assert!(c.current_value.is_empty());
        assert_eq!(c.previous_value, None);
        assert_eq!(c.operation, None);
    }

    #[test]
    fn input_during_pending_operation_does_not_overwrite_display() {
        let mut c = calc();
        run(
            &mut c,
            &[
                Message::InputNumber("5".into()),
                Message::Operation(Operation::Add),
                Message::InputNumber("3".into()),
            ],
        );
        // Display still shows the buffered "5" until Calculate runs.
        assert_eq!(c.display, "5");
        assert_eq!(c.current_value, "3");
    }

    #[test]
    fn fractional_results_format_with_decimal() {
        let mut c = calc();
        let r = run(
            &mut c,
            &[
                Message::InputNumber("1".into()),
                Message::Operation(Operation::Divide),
                Message::InputNumber("4".into()),
                Message::Calculate,
            ],
        );
        assert_eq!(r, "0.25");
    }

    #[test]
    fn operation_display_format() {
        assert_eq!(format!("{}", Operation::Add), "+");
        assert_eq!(format!("{}", Operation::Subtract), "-");
        assert_eq!(format!("{}", Operation::Multiply), "*");
        assert_eq!(format!("{}", Operation::Divide), "/");
    }

    #[test]
    fn provable_contract_holds() {
        check_calculate_no_op_when_no_operation().expect("contract should hold");
    }
}
