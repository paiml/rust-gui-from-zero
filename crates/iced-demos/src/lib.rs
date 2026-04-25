//! iced demos — logic modules.
//!
//! Each demo binary in `src/bin/*.rs` wraps a pure logic type from one of the
//! modules below. Splitting state from view lets these state machines be
//! exhaustively tested without booting the iced runtime, and lets the
//! workspace hit 100% coverage on its testable surface.
//!
//! # Provable contracts
//!
//! Every demo carries a named `Provable contract` in its bin docstring; the
//! contract is checked at startup with [`contracts::assert_invariant!`].

pub mod calculator;
pub mod counter;
pub mod hello;
pub mod minimal;
pub mod todo;
pub mod todo_persist;
