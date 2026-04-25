//! `gtk-random-password-step3-visibility` — GTK4 password generator
//! with a length slider and a show/hide toggle.
//!
//! **Provable contract:** `PWGEN_LENGTH` / `PWGEN_CHARSET` — toggling
//! the visibility `Switch` only flips `Entry::set_visibility`; the
//! generated password itself is still produced by the pure
//! `generate_password` function and respects the length / charset
//! invariants.

#![allow(clippy::wildcard_imports, clippy::too_many_lines)]
use contracts::assert_invariant;
use gtk::glib;
use gtk::prelude::*;
use gtk::{
    Application, ApplicationWindow, Box as GtkBox, Button, Entry, Label, Orientation, Scale, Switch,
};
use gtk_demos::password::{
    check_password_invariants, generate_password, DEFAULT_LENGTH, MAX_LENGTH, MIN_LENGTH,
};
use std::rc::Rc;

const APP_ID: &str = "org.example.PasswordGeneratorStep3";

fn main() {
    assert_invariant!(PWGEN_CONTRACT_HOLDS, check_password_invariants().is_ok());
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run();
}

#[allow(clippy::cast_precision_loss)]
fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Password Generator — Visibility")
        .default_width(800)
        .default_height(600)
        .build();
    let vbox = GtkBox::new(Orientation::Vertical, 20);
    vbox.set_margin_top(20);
    vbox.set_margin_bottom(20);
    vbox.set_margin_start(20);
    vbox.set_margin_end(20);
    let password_entry = Entry::builder().editable(false).visibility(false).build();
    let length_scale = Scale::with_range(
        Orientation::Horizontal,
        MIN_LENGTH as f64,
        MAX_LENGTH as f64,
        1.0,
    );
    length_scale.set_value(DEFAULT_LENGTH as f64);
    length_scale.set_draw_value(true);
    let visibility_row = GtkBox::new(Orientation::Horizontal, 10);
    let visibility_label = Label::new(Some("Show password:"));
    let visibility_switch = Switch::builder().active(false).build();
    visibility_row.append(&visibility_label);
    visibility_row.append(&visibility_switch);
    let generate_button = Button::builder().label("Generate Password").build();
    let password_entry = Rc::new(password_entry);
    let length_scale = Rc::new(length_scale);
    let entry_for_button = Rc::clone(&password_entry);
    let scale_for_button = Rc::clone(&length_scale);
    generate_button.connect_clicked(move |_| {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let len = scale_for_button.value() as usize;
        let pw = generate_password(&mut rand::thread_rng(), len);
        entry_for_button.set_text(&pw);
    });
    let entry_for_switch = Rc::clone(&password_entry);
    visibility_switch.connect_state_set(move |_, state| {
        entry_for_switch.set_visibility(state);
        glib::Propagation::Proceed
    });
    vbox.append(&*password_entry);
    vbox.append(&*length_scale);
    vbox.append(&visibility_row);
    vbox.append(&generate_button);
    window.set_child(Some(&vbox));
    window.present();
}
