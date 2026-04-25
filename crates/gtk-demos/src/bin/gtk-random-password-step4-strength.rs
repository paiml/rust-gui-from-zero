//! `gtk-random-password-step4-strength` — GTK4 password generator with
//! length slider, visibility switch, strength indicator, and clipboard
//! copy.
//!
//! **Provable contract:** `PWGEN_LENGTH` / `PWGEN_CHARSET` /
//! `PWGEN_STRENGTH_RANGE` — the indicator reads `password_strength`,
//! whose output stays in `[0.0, 1.0]`; `ProgressBar::set_fraction`
//! therefore receives a valid fraction for any generated password.

#![allow(clippy::wildcard_imports, clippy::too_many_lines)]
use contracts::assert_invariant;
use gtk::glib;
use gtk::prelude::*;
use gtk::{
    Application, ApplicationWindow, Box as GtkBox, Button, Entry, Label, Orientation, ProgressBar,
    Scale, Switch,
};
use gtk_demos::password::{
    check_password_invariants, classify_strength, generate_password, password_strength,
    DEFAULT_LENGTH, MAX_LENGTH, MIN_LENGTH,
};
use std::rc::Rc;

const APP_ID: &str = "org.example.PasswordGeneratorStep4";

fn main() {
    assert_invariant!(PWGEN_CONTRACT_HOLDS, check_password_invariants().is_ok());
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run();
}

fn update_strength(progress: &ProgressBar, label: &Label, password: &str) {
    let score = password_strength(password);
    progress.set_fraction(score);
    label.set_text(classify_strength(score).label());
}

#[allow(clippy::cast_precision_loss)]
fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Password Generator — Strength")
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
    let strength_bar = ProgressBar::new();
    let strength_label = Label::new(Some("Weak"));
    let button_row = GtkBox::new(Orientation::Horizontal, 10);
    let generate_button = Button::builder().label("Generate Password").build();
    let copy_button = Button::builder().label("Copy Password").build();
    button_row.append(&generate_button);
    button_row.append(&copy_button);

    let password_entry = Rc::new(password_entry);
    let length_scale = Rc::new(length_scale);
    let strength_bar = Rc::new(strength_bar);
    let strength_label = Rc::new(strength_label);

    let entry_for_button = Rc::clone(&password_entry);
    let scale_for_button = Rc::clone(&length_scale);
    let bar_for_button = Rc::clone(&strength_bar);
    let label_for_button = Rc::clone(&strength_label);
    generate_button.connect_clicked(move |_| {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let len = scale_for_button.value() as usize;
        let pw = generate_password(&mut rand::thread_rng(), len);
        entry_for_button.set_text(&pw);
        update_strength(&bar_for_button, &label_for_button, &pw);
    });

    let entry_for_switch = Rc::clone(&password_entry);
    visibility_switch.connect_state_set(move |_, state| {
        entry_for_switch.set_visibility(state);
        glib::Propagation::Proceed
    });

    let entry_for_copy = Rc::clone(&password_entry);
    copy_button.connect_clicked(move |btn| {
        let text = entry_for_copy.text();
        if !text.is_empty() {
            btn.clipboard().set_text(&text);
        }
    });

    vbox.append(&*password_entry);
    vbox.append(&*length_scale);
    vbox.append(&visibility_row);
    vbox.append(&*strength_bar);
    vbox.append(&*strength_label);
    vbox.append(&button_row);
    window.set_child(Some(&vbox));
    window.present();
}
