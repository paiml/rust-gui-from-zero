//! relm4 demos — logic modules for unit testing.
//!
//! View / event-loop code lives in `src/bin/*.rs`; this lib carries only the
//! Simon-Says state machine (sequence generation, color comparison) so the
//! demo's invariants can be tested without a GTK display.

pub mod simon;
