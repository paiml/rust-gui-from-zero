//! Pure config struct for the GTK hello demo. View code lives in `bin/`.

use contracts::ContractError;

#[derive(Debug, Clone)]
pub struct HelloConfig {
    pub width: i32,
    pub height: i32,
    pub title: String,
    pub markup: String,
    pub app_id: String,
}

impl Default for HelloConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl HelloConfig {
    #[must_use]
    pub fn new() -> Self {
        Self {
            width: 800,
            height: 600,
            title: "Hello World".into(),
            markup: "<span size='50000' weight='bold'>Hello, World!</span>".into(),
            app_id: "org.example.HelloWorld".into(),
        }
    }
}

/// Validate an arbitrary [`HelloConfig`] — extracted so negative-path tests
/// can exercise every error branch.
///
/// # Errors
///
/// Returns [`contracts::ContractError`] if any field is invalid.
pub fn validate(cfg: &HelloConfig) -> Result<(), ContractError> {
    if cfg.width <= 0 || cfg.height <= 0 {
        return Err(ContractError {
            name: "GTK_HELLO_VALID",
            message: format!("non-positive dimensions {}x{}", cfg.width, cfg.height),
        });
    }
    if cfg.title.is_empty() || cfg.markup.is_empty() || cfg.app_id.is_empty() {
        return Err(ContractError {
            name: "GTK_HELLO_VALID",
            message: "empty title, markup, or app_id".into(),
        });
    }
    Ok(())
}

/// Provable contract: `HelloConfig::new()` produces positive dimensions and
/// non-empty title / markup / `app_id`.
///
/// # Errors
///
/// Returns [`contracts::ContractError`] if any field is invalid.
pub fn check_hello_config_valid() -> Result<(), ContractError> {
    validate(&HelloConfig::new())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        check_hello_config_valid().unwrap();
    }

    #[test]
    fn default_config_fields_match() {
        let cfg = HelloConfig::default();
        assert_eq!(cfg.width, 800);
        assert_eq!(cfg.height, 600);
        assert_eq!(cfg.title, "Hello World");
        assert!(cfg.markup.contains("Hello, World!"));
        assert_eq!(cfg.app_id, "org.example.HelloWorld");
    }

    #[test]
    fn validate_rejects_zero_width() {
        let mut cfg = HelloConfig::new();
        cfg.width = 0;
        let err = validate(&cfg).unwrap_err();
        assert_eq!(err.name, "GTK_HELLO_VALID");
    }

    #[test]
    fn validate_rejects_negative_height() {
        let mut cfg = HelloConfig::new();
        cfg.height = -1;
        assert!(validate(&cfg).is_err());
    }

    #[test]
    fn validate_rejects_empty_title() {
        let mut cfg = HelloConfig::new();
        cfg.title = String::new();
        assert!(validate(&cfg).is_err());
    }

    #[test]
    fn validate_rejects_empty_markup() {
        let mut cfg = HelloConfig::new();
        cfg.markup = String::new();
        assert!(validate(&cfg).is_err());
    }

    #[test]
    fn validate_rejects_empty_app_id() {
        let mut cfg = HelloConfig::new();
        cfg.app_id = String::new();
        assert!(validate(&cfg).is_err());
    }
}
