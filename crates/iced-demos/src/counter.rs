//! Pure counter state for the `iced-button-counter` demo.

/// Monotonically-increasing counter — bumped by one on every `Increment`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Counter {
    pub count: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Message {
    Increment,
}

impl Counter {
    /// Construct a counter at zero.
    #[must_use]
    pub const fn new() -> Self {
        Self { count: 0 }
    }

    /// Apply a [`Message`] and return the new count.
    pub fn apply(&mut self, message: Message) -> i32 {
        match message {
            Message::Increment => {
                self.count = self.count.saturating_add(1);
            }
        }
        self.count
    }
}

/// Provable contract: every Increment strictly raises the count by 1
/// (until `i32::MAX`, where saturation kicks in).
///
/// # Errors
///
/// Returns [`contracts::ContractError`] if the invariant is violated.
pub fn check_increment_monotonic() -> Result<(), contracts::ContractError> {
    let mut c = Counter::new();
    let before = c.count;
    let after = c.apply(Message::Increment);
    contracts::check(
        "COUNTER_INCREMENT_MONOTONIC",
        after == before + 1,
        "Increment must raise the count by exactly 1",
    )
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn new_starts_at_zero() {
        assert_eq!(Counter::new().count, 0);
    }

    #[test]
    fn default_starts_at_zero() {
        assert_eq!(Counter::default(), Counter::new());
    }

    #[test]
    fn increment_raises_by_one() {
        let mut c = Counter::new();
        assert_eq!(c.apply(Message::Increment), 1);
        assert_eq!(c.apply(Message::Increment), 2);
    }

    #[test]
    fn increment_saturates_at_i32_max() {
        let mut c = Counter { count: i32::MAX };
        assert_eq!(c.apply(Message::Increment), i32::MAX);
    }

    #[test]
    fn provable_contract_holds() {
        check_increment_monotonic().expect("contract should hold");
    }
}
