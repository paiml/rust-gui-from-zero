//! `relm4-simon` — minimal Simon-Says skeleton: four color buttons,
//! each press is logged to stdout.
//!
//! **Provable contract:** `SIMON_INDEX_BOUNDS` — even though this
//! skeleton ignores game state, the underlying `Color` enum + RNG
//! sampler are exercised by `check_simon_invariants` at startup, so
//! the binary refuses to launch if those primitives are broken.

#![allow(clippy::wildcard_imports, clippy::too_many_lines)]
use contracts::assert_invariant;
use gtk::prelude::*;
use relm4::prelude::*;
use relm4_demos::simon::{check_simon_invariants, Color};

#[derive(Debug)]
struct App;

#[derive(Debug)]
enum Msg {
    Pressed(Color),
}

#[relm4::component]
impl SimpleComponent for App {
    type Init = ();
    type Input = Msg;
    type Output = ();

    view! {
        gtk::ApplicationWindow {
            set_title: Some("Simon Says — Skeleton"),
            set_default_size: (300, 300),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 5,
                set_margin_all: 10,

                gtk::Button {
                    set_label: "Red",
                    connect_clicked => Msg::Pressed(Color::Red),
                },
                gtk::Button {
                    set_label: "Green",
                    connect_clicked => Msg::Pressed(Color::Green),
                },
                gtk::Button {
                    set_label: "Blue",
                    connect_clicked => Msg::Pressed(Color::Blue),
                },
                gtk::Button {
                    set_label: "Yellow",
                    connect_clicked => Msg::Pressed(Color::Yellow),
                },
            }
        }
    }

    fn init(
        (): Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = App;
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            Msg::Pressed(color) => println!("{} button clicked", color.label()),
        }
    }
}

fn main() {
    assert_invariant!(SIMON_CONTRACT_HOLDS, check_simon_invariants().is_ok());
    let app = RelmApp::new("relm4.example.simon_says.skeleton");
    app.run::<App>(());
}
