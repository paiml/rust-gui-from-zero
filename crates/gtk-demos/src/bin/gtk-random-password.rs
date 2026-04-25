//! `gtk-random-password` — GTK4 random password generator (12-char fixed).
//!
//! **Provable contract:** `PWGEN_LENGTH` / `PWGEN_CHARSET` — generated
//! password matches the requested length (clamped) and is alphanumeric only.

#![allow(clippy::wildcard_imports, clippy::too_many_lines)]
use contracts::assert_invariant;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box as GtkBox, Button, Entry, Orientation};
use gtk_demos::password::{check_password_invariants, generate_password, DEFAULT_LENGTH};
use std::rc::Rc;

const APP_ID: &str = "org.example.PasswordGenerator";

fn main() {
    assert_invariant!(PWGEN_CONTRACT_HOLDS, check_password_invariants().is_ok());
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Password Generator")
        .default_width(800)
        .default_height(600)
        .build();
    let vbox = GtkBox::new(Orientation::Vertical, 20);
    vbox.set_margin_top(20);
    vbox.set_margin_bottom(20);
    vbox.set_margin_start(20);
    vbox.set_margin_end(20);
    let password_entry = Entry::builder().editable(false).build();
    let generate_button = Button::builder().label("Generate Password").build();
    let password_entry = Rc::new(password_entry);
    let entry_clone = Rc::clone(&password_entry);
    generate_button.connect_clicked(move |_| {
        let pw = generate_password(&mut rand::thread_rng(), DEFAULT_LENGTH);
        entry_clone.set_text(&pw);
    });
    vbox.append(&*password_entry);
    vbox.append(&generate_button);
    window.set_child(Some(&vbox));
    window.present();
}
