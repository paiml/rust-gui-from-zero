//! egui / eframe demos — logic modules for unit testing.
//!
//! View / event-loop code lives in `src/bin/*.rs`; this lib carries only the
//! quiz-state logic (loading questions, scoring answers) so the demo's
//! invariants can be tested headlessly.

pub mod quiz;
