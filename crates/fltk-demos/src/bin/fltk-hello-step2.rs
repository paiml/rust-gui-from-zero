//! `fltk-hello-step2` — same as `fltk-hello` but with named constants
//! pulled out of the hard-coded literals.
//!
//! **Provable contract:** `FLTK_HELLO_VALID` — default window has
//! positive dimensions and non-empty labels.

#![allow(clippy::wildcard_imports, clippy::too_many_lines)]
use contracts::assert_invariant;
use fltk::{prelude::*, *};
use fltk_demos::hello::{check_hello_config_valid, HelloConfig};

fn main() {
    assert_invariant!(
        FLTK_HELLO_CONTRACT_HOLDS,
        check_hello_config_valid().is_ok()
    );
    let cfg = HelloConfig::new();
    let app = app::App::default();
    let mut wind = window::Window::new(100, 100, cfg.width, cfg.height, cfg.title.as_str());
    let mut frame = frame::Frame::new(0, 0, cfg.width, cfg.height, cfg.message.as_str());
    frame.set_align(enums::Align::Center);
    wind.end();
    wind.show();
    if let Err(e) = app.run() {
        eprintln!("fltk-hello-step2 exited: {e}");
    }
}
