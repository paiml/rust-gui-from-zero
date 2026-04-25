//! GTK4 demos — logic modules for unit testing.
//!
//! View / event-loop code lives in `src/bin/*.rs`; this lib carries only the
//! pure logic (password generation, character-class strength scoring) so the
//! demo's invariants can be tested without a GTK display.

pub mod hello;
pub mod password;
