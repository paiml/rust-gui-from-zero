//! Window/title configuration for the `fltk-hello*` demos.
//!
//! All three hello demos render a fixed-size window with a centered text
//! label. This module owns the size/title constants so the binaries are
//! pure view code.

#[derive(Debug, Clone)]
pub struct HelloConfig {
    pub width: i32,
    pub height: i32,
    pub title: String,
    pub message: String,
}

impl Default for HelloConfig {
    fn default() -> Self {
        Self {
            width: 400,
            height: 300,
            title: "[Window] Hello World".to_string(),
            message: "Hello World!".to_string(),
        }
    }
}

impl HelloConfig {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

/// Provable contract: the default hello window has positive dimensions
/// and non-empty title/message.
///
/// # Errors
///
/// Returns [`contracts::ContractError`] if any field violates the
/// invariant.
pub fn check_hello_config_valid() -> Result<(), contracts::ContractError> {
    let cfg = HelloConfig::new();
    contracts::check(
        "FLTK_HELLO_VALID",
        cfg.width > 0 && cfg.height > 0 && !cfg.title.is_empty() && !cfg.message.is_empty(),
        "hello window dimensions must be positive and labels non-empty",
    )
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn default_dimensions_positive() {
        let cfg = HelloConfig::new();
        assert!(cfg.width > 0);
        assert!(cfg.height > 0);
    }

    #[test]
    fn default_strings_non_empty() {
        let cfg = HelloConfig::new();
        assert!(!cfg.title.is_empty());
        assert!(!cfg.message.is_empty());
    }

    #[test]
    fn default_matches_legacy_demo() {
        let cfg = HelloConfig::default();
        assert_eq!(cfg.width, 400);
        assert_eq!(cfg.height, 300);
        assert_eq!(cfg.title, "[Window] Hello World");
        assert_eq!(cfg.message, "Hello World!");
    }

    #[test]
    fn provable_contract_holds() {
        check_hello_config_valid().expect("contract should hold");
    }
}
