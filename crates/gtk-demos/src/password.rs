//! Pure password-generator logic — no GTK, no system clock.
//!
//! View binaries inject a `rand::thread_rng()` at runtime; tests inject a
//! seeded `StdRng` so the assertions are deterministic.

use contracts::ContractError;
use rand::distributions::{Alphanumeric, DistString};
use rand::Rng;

pub const DEFAULT_LENGTH: usize = 12;
pub const MIN_LENGTH: usize = 1;
pub const MAX_LENGTH: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Strength {
    Weak,
    Moderate,
    Strong,
    VeryStrong,
}

impl Strength {
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Weak => "Weak",
            Self::Moderate => "Moderate",
            Self::Strong => "Strong",
            Self::VeryStrong => "Very Strong",
        }
    }
}

/// Generate an alphanumeric password of `length` characters using the given RNG.
///
/// The length is clamped to `[MIN_LENGTH, MAX_LENGTH]` so view code never has
/// to validate slider input.
pub fn generate_password<R: Rng>(rng: &mut R, length: usize) -> String {
    let len = length.clamp(MIN_LENGTH, MAX_LENGTH);
    Alphanumeric.sample_string(rng, len)
}

/// Score a password's strength as a fraction in `[0.0, 1.0]`.
///
/// Combines a length factor (capped at 30 chars) and a character-class
/// variety factor (lowercase / uppercase / digit / non-alphanumeric).
#[must_use]
pub fn password_strength(password: &str) -> f64 {
    if password.is_empty() {
        return 0.0;
    }
    #[allow(clippy::cast_precision_loss)]
    let length = password.len() as f64;
    let has_lower = password.chars().any(|c| c.is_ascii_lowercase());
    let has_upper = password.chars().any(|c| c.is_ascii_uppercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());

    #[allow(clippy::cast_precision_loss)]
    let variety = [has_lower, has_upper, has_digit, has_special]
        .iter()
        .filter(|&&x| x)
        .count() as f64
        / 4.0;
    let length_score = (length / 30.0).min(1.0);
    (length_score * 0.7 + variety * 0.3).min(1.0)
}

#[must_use]
pub fn classify_strength(score: f64) -> Strength {
    if score <= 0.25 {
        Strength::Weak
    } else if score <= 0.5 {
        Strength::Moderate
    } else if score <= 0.75 {
        Strength::Strong
    } else {
        Strength::VeryStrong
    }
}

/// Validate a single (requested length, generated password) pair.
///
/// Extracted so negative-path tests can exercise every error branch with
/// synthetic data. The strength check accepts a precomputed score so
/// out-of-range values can be exercised directly.
///
/// # Errors
///
/// Returns [`contracts::ContractError`] when any contract is violated.
pub fn validate_sample(requested: usize, pw: &str, strength: f64) -> Result<(), ContractError> {
    let expected = requested.clamp(MIN_LENGTH, MAX_LENGTH);
    if pw.len() != expected {
        return Err(ContractError {
            name: "PWGEN_LENGTH",
            message: format!(
                "requested {requested} (clamped {expected}), got {} chars",
                pw.len()
            ),
        });
    }
    if !pw.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(ContractError {
            name: "PWGEN_CHARSET",
            message: "non-ASCII-alphanumeric character produced".into(),
        });
    }
    if !(0.0..=1.0).contains(&strength) {
        return Err(ContractError {
            name: "PWGEN_STRENGTH_RANGE",
            message: format!("strength {strength} outside [0.0, 1.0]"),
        });
    }
    Ok(())
}

/// Provable contract: `generate_password` always returns a string of the
/// requested length (clamped to `[MIN_LENGTH, MAX_LENGTH]`), every character
/// is ASCII alphanumeric, and `password_strength` returns a value in
/// `[0.0, 1.0]`.
///
/// # Errors
///
/// Returns [`contracts::ContractError`] if any property is violated.
pub fn check_password_invariants() -> Result<(), ContractError> {
    use rand::SeedableRng;
    let mut rng = rand::rngs::StdRng::seed_from_u64(0x00C0_FFEE);
    for &requested in &[0usize, 1, 8, 12, 30, 64, 1024] {
        let pw = generate_password(&mut rng, requested);
        let s = password_strength(&pw);
        validate_sample(requested, &pw, s)?;
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    fn rng() -> StdRng {
        StdRng::seed_from_u64(42)
    }

    #[test]
    fn generate_password_default_length() {
        let mut r = rng();
        let pw = generate_password(&mut r, DEFAULT_LENGTH);
        assert_eq!(pw.len(), DEFAULT_LENGTH);
    }

    #[test]
    fn generate_password_clamps_zero_to_min() {
        let mut r = rng();
        let pw = generate_password(&mut r, 0);
        assert_eq!(pw.len(), MIN_LENGTH);
    }

    #[test]
    fn generate_password_clamps_huge_to_max() {
        let mut r = rng();
        let pw = generate_password(&mut r, 99_999);
        assert_eq!(pw.len(), MAX_LENGTH);
    }

    #[test]
    fn generate_password_only_alphanumeric() {
        let mut r = rng();
        let pw = generate_password(&mut r, 64);
        assert!(pw.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn generate_password_seeded_deterministic() {
        let mut a = StdRng::seed_from_u64(7);
        let mut b = StdRng::seed_from_u64(7);
        assert_eq!(generate_password(&mut a, 16), generate_password(&mut b, 16));
    }

    #[test]
    fn empty_password_strength_is_zero() {
        assert!((password_strength("") - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn short_lowercase_is_weak() {
        let s = password_strength("abc");
        assert!(s < 0.25, "{s}");
        assert_eq!(classify_strength(s), Strength::Weak);
    }

    #[test]
    fn mixed_case_long_is_strong() {
        let s = password_strength("AbCdEfGh1234");
        assert!(s > 0.25 && s <= 0.75, "{s}");
    }

    #[test]
    fn with_special_chars_is_very_strong() {
        let s = password_strength("AbCdEfGh1234!@#$%^&*()ZxYwVuTs");
        assert!(s > 0.75, "{s}");
        assert_eq!(classify_strength(s), Strength::VeryStrong);
    }

    #[test]
    fn classify_strength_boundaries() {
        assert_eq!(classify_strength(0.0), Strength::Weak);
        assert_eq!(classify_strength(0.25), Strength::Weak);
        assert_eq!(classify_strength(0.26), Strength::Moderate);
        assert_eq!(classify_strength(0.5), Strength::Moderate);
        assert_eq!(classify_strength(0.51), Strength::Strong);
        assert_eq!(classify_strength(0.75), Strength::Strong);
        assert_eq!(classify_strength(0.76), Strength::VeryStrong);
        assert_eq!(classify_strength(1.0), Strength::VeryStrong);
    }

    #[test]
    fn strength_labels() {
        assert_eq!(Strength::Weak.label(), "Weak");
        assert_eq!(Strength::Moderate.label(), "Moderate");
        assert_eq!(Strength::Strong.label(), "Strong");
        assert_eq!(Strength::VeryStrong.label(), "Very Strong");
    }

    #[test]
    fn invariants_hold() {
        check_password_invariants().unwrap();
    }

    #[test]
    fn validate_sample_accepts_correct_length() {
        validate_sample(8, "abcd1234", 0.5).unwrap();
    }

    #[test]
    fn validate_sample_rejects_wrong_length() {
        let err = validate_sample(8, "ab", 0.5).unwrap_err();
        assert_eq!(err.name, "PWGEN_LENGTH");
    }

    #[test]
    fn validate_sample_rejects_non_alphanumeric() {
        let err = validate_sample(8, "abcd!@#$", 0.5).unwrap_err();
        assert_eq!(err.name, "PWGEN_CHARSET");
    }

    #[test]
    fn validate_sample_clamps_zero_to_min() {
        validate_sample(0, "x", 0.0).unwrap();
        let err = validate_sample(0, "", 0.0).unwrap_err();
        assert_eq!(err.name, "PWGEN_LENGTH");
    }

    #[test]
    fn validate_sample_rejects_strength_above_one() {
        let err = validate_sample(8, "abcd1234", 2.0).unwrap_err();
        assert_eq!(err.name, "PWGEN_STRENGTH_RANGE");
    }

    #[test]
    fn validate_sample_rejects_strength_below_zero() {
        let err = validate_sample(8, "abcd1234", -0.5).unwrap_err();
        assert_eq!(err.name, "PWGEN_STRENGTH_RANGE");
    }
}
