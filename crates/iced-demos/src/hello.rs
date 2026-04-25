//! Greeting strings for the `iced-hello` demo.
//!
//! There is no state machine — the only logic worth testing is the
//! invariant that the title and body are non-empty constant text.

/// Window title shown by `iced-hello`.
pub const TITLE: &str = "Hello, Iced!";

/// Body text rendered inside the window.
pub const BODY: &str = "Hello, Iced!";

/// Provable contract: title and body are non-empty (the window must show
/// something — a blank window would fail QA).
///
/// # Errors
///
/// Returns [`contracts::ContractError`] if either string is empty.
pub fn check_greeting_non_empty() -> Result<(), contracts::ContractError> {
    contracts::check(
        "HELLO_GREETING_NON_EMPTY",
        !TITLE.is_empty() && !BODY.is_empty(),
        "TITLE and BODY must be non-empty",
    )
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn title_is_non_empty() {
        assert!(!TITLE.is_empty());
    }

    #[test]
    fn body_is_non_empty() {
        assert!(!BODY.is_empty());
    }

    #[test]
    fn provable_contract_holds() {
        check_greeting_non_empty().expect("contract should hold");
    }
}
