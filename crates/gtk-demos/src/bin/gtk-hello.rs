//! `gtk-hello` — minimal GTK4 hello-world window.
//!
//! **Provable contract:** `GTK_HELLO_VALID` — default config has positive
//! dimensions and non-empty title / markup / `app_id`.

#![allow(clippy::wildcard_imports, clippy::too_many_lines)]
use contracts::assert_invariant;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Label};
use gtk_demos::hello::{check_hello_config_valid, HelloConfig};

fn main() {
    assert_invariant!(GTK_HELLO_CONTRACT_HOLDS, check_hello_config_valid().is_ok());
    let cfg = HelloConfig::new();
    let app = Application::builder().application_id(&cfg.app_id).build();
    app.connect_activate(move |app| {
        let label = Label::builder()
            .label(&cfg.markup)
            .use_markup(true)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Center)
            .build();
        let window = ApplicationWindow::builder()
            .application(app)
            .title(&cfg.title)
            .default_width(cfg.width)
            .default_height(cfg.height)
            .child(&label)
            .build();
        window.present();
    });
    app.run();
}
