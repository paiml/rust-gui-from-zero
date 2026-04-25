//! Shared provable-contract scaffolding for the rust-gui-from-zero workspace.
//!
//! Every demo binary in this workspace declares a named [`Provable contract`]
//! in its module docstring and verifies it at startup with
//! [`assert_invariant!`]. The macro is a thin wrapper over `assert!` that
//! prefixes failures with `[contract:<NAME>]` so a panic in CI or a learner's
//! terminal points at the violated invariant by name.
//!
//! See `rust-de-specialization` memory `feedback-provable-contracts-required.md`
//! for the policy that produced this crate.

#![doc(html_root_url = "https://docs.rs/contracts/0.1.0")]

/// Assert a named, runtime-checkable invariant.
///
/// On failure the panic message embeds the contract name, the file/line, and
/// (optionally) a formatted explanation, e.g.
/// `[contract:CALC_BINARY_OP_TOTAL] divisor cannot be zero`.
///
/// # Examples
///
/// ```
/// use contracts::assert_invariant;
/// let xs = [1, 2, 3];
/// assert_invariant!(EXAMPLE_NON_EMPTY, !xs.is_empty(),
///     "demo input must contain at least one element");
/// ```
#[macro_export]
macro_rules! assert_invariant {
    ($name:ident, $cond:expr) => {
        ::std::assert!(
            $cond,
            "[contract:{}] {} ({}:{})",
            stringify!($name),
            stringify!($cond),
            file!(),
            line!()
        );
    };
    ($name:ident, $cond:expr, $($arg:tt)+) => {
        ::std::assert!(
            $cond,
            "[contract:{}] {} ({}:{})",
            stringify!($name),
            format_args!($($arg)+),
            file!(),
            line!()
        );
    };
}

/// Verify a contract holds, returning `Result` instead of panicking.
///
/// Useful when the demo wants to surface a contract violation through the GUI
/// (e.g. an error dialog) rather than aborting the process. The returned
/// [`ContractError`] carries the same structured payload as a panicking
/// [`assert_invariant!`].
///
/// # Errors
///
/// Returns [`ContractError`] when `cond` is false.
#[inline]
pub fn check(name: &'static str, cond: bool, msg: &str) -> Result<(), ContractError> {
    if cond {
        Ok(())
    } else {
        Err(ContractError {
            name,
            message: msg.to_owned(),
        })
    }
}

/// Structured contract violation, returned by [`check`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractError {
    /// Contract name (e.g. `CALC_BINARY_OP_TOTAL`).
    pub name: &'static str,
    /// Human-readable explanation of the violation.
    pub message: String,
}

impl std::fmt::Display for ContractError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[contract:{}] {}", self.name, self.message)
    }
}

impl std::error::Error for ContractError {}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn assert_invariant_passes_when_true() {
        assert_invariant!(EXAMPLE, 1 + 1 == 2);
    }

    // The two panic tests below keep their bodies on a single line so the
    // closing brace shares a line with the panic-emitting macro call. With
    // `#[should_panic]` the function never reaches its closing brace, which
    // would otherwise show as uncovered under cargo-llvm-cov. `#[rustfmt::skip]`
    // prevents rustfmt from re-expanding them.

    #[test]
    #[should_panic(expected = "[contract:EXAMPLE]")]
    #[rustfmt::skip]
    fn assert_invariant_panics_when_false() { assert_invariant!(EXAMPLE, false); }

    #[test]
    #[should_panic(expected = "[contract:EXAMPLE] divisor was 0")]
    #[rustfmt::skip]
    fn assert_invariant_with_message() { let d = 0; assert_invariant!(EXAMPLE, d != 0, "divisor was {d}"); }

    #[test]
    fn check_returns_ok_when_true() {
        assert_eq!(check("E", true, "ignored"), Ok(()));
    }

    #[test]
    fn check_returns_err_when_false() {
        let err = check("E", false, "boom").expect_err("expected ContractError");
        assert_eq!(err.name, "E");
        assert_eq!(err.message, "boom");
    }

    #[test]
    fn contract_error_display_includes_name() {
        let err = ContractError {
            name: "MY_RULE",
            message: "violated".into(),
        };
        assert_eq!(format!("{err}"), "[contract:MY_RULE] violated");
    }
}
