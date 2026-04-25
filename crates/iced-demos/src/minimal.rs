//! Pure +/-/clear state for the `minimal-iced` demo.

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MinimalApp {
    pub value: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Message {
    Increment,
    Decrement,
    Clear,
}

impl MinimalApp {
    #[must_use]
    pub const fn new() -> Self {
        Self { value: 0 }
    }

    pub fn apply(&mut self, message: Message) -> i32 {
        match message {
            Message::Increment => self.value = self.value.saturating_add(1),
            Message::Decrement => self.value = self.value.saturating_sub(1),
            Message::Clear => self.value = 0,
        }
        self.value
    }
}

/// Provable contract: `Clear` always resets to 0, regardless of prior state.
///
/// # Errors
///
/// Returns [`contracts::ContractError`] if Clear leaves a non-zero value.
pub fn check_clear_resets_to_zero() -> Result<(), contracts::ContractError> {
    let mut a = MinimalApp { value: 999 };
    a.apply(Message::Clear);
    contracts::check(
        "MINIMAL_CLEAR_RESETS",
        a.value == 0,
        "Clear must always reset value to 0",
    )
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn new_is_zero() {
        assert_eq!(MinimalApp::new().value, 0);
    }

    #[test]
    fn default_is_zero() {
        assert_eq!(MinimalApp::default(), MinimalApp::new());
    }

    #[test]
    fn increment_adds_one() {
        let mut a = MinimalApp::new();
        assert_eq!(a.apply(Message::Increment), 1);
    }

    #[test]
    fn decrement_subtracts_one() {
        let mut a = MinimalApp::new();
        assert_eq!(a.apply(Message::Decrement), -1);
    }

    #[test]
    fn clear_from_positive_resets_to_zero() {
        let mut a = MinimalApp { value: 42 };
        assert_eq!(a.apply(Message::Clear), 0);
    }

    #[test]
    fn clear_from_negative_resets_to_zero() {
        let mut a = MinimalApp { value: -42 };
        assert_eq!(a.apply(Message::Clear), 0);
    }

    #[test]
    fn increment_saturates() {
        let mut a = MinimalApp { value: i32::MAX };
        assert_eq!(a.apply(Message::Increment), i32::MAX);
    }

    #[test]
    fn decrement_saturates() {
        let mut a = MinimalApp { value: i32::MIN };
        assert_eq!(a.apply(Message::Decrement), i32::MIN);
    }

    #[test]
    fn provable_contract_holds() {
        check_clear_resets_to_zero().expect("contract should hold");
    }
}
