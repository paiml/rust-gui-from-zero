//! FLTK demos — logic modules for unit testing.
//!
//! View / event-loop code lives in `src/bin/*.rs`; this lib carries only
//! the pure simulation logic (hello config, pong physics, high-score
//! persistence) so each demo's invariants can be tested without an FLTK
//! display.

pub mod hello;
pub mod highscore;
pub mod pong;
