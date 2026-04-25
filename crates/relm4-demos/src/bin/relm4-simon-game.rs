//! `relm4-simon-game` — full Simon-Says game with sequence growth,
//! game-over detection, and "New Game" reset.
//!
//! **Provable contract:** `SIMON_INDEX_BOUNDS` / `SIMON_GAME_OVER`
//! / `SIMON_GAME_OVER_FROZEN` — the state machine in
//! `relm4_demos::simon` is exercised by `check_simon_invariants` at
//! startup; the view code only forwards `Color` presses into
//! `SimonGame::press`, which preserves those invariants by
//! construction.

#![allow(clippy::wildcard_imports, clippy::too_many_lines)]
use contracts::assert_invariant;
use gtk::prelude::*;
use rand::rngs::ThreadRng;
use relm4::prelude::*;
use relm4_demos::simon::{check_simon_invariants, Color, PressOutcome, SimonGame};

#[derive(Debug)]
struct App {
    game: SimonGame,
    status: String,
    rng: ThreadRng,
}

#[derive(Debug)]
enum Msg {
    Pressed(Color),
    NewGame,
}

#[relm4::component]
impl SimpleComponent for App {
    type Init = ();
    type Input = Msg;
    type Output = ();

    view! {
        gtk::ApplicationWindow {
            set_title: Some("Simon Says"),
            set_default_size: (320, 360),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 5,
                set_margin_all: 10,

                gtk::Label {
                    #[watch]
                    set_label: &model.status,
                    set_margin_all: 5,
                },

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

                gtk::Button {
                    set_label: "New Game",
                    connect_clicked => Msg::NewGame,
                },
            }
        }
    }

    fn init(
        (): Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = App {
            game: SimonGame::new(),
            status: "Press 'New Game' to start.".into(),
            rng: rand::thread_rng(),
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            Msg::Pressed(color) => {
                let outcome = self.game.press(color, &mut self.rng);
                self.status = match outcome {
                    PressOutcome::Idle => "Press 'New Game' to start.".into(),
                    PressOutcome::Correct => {
                        format!("Correct — sequence length {}", self.game.level())
                    }
                    PressOutcome::RoundComplete => {
                        format!("Round complete! New sequence length: {}", self.game.level())
                    }
                    PressOutcome::Wrong => {
                        format!("Game over. Final sequence length: {}", self.game.level())
                    }
                    PressOutcome::AlreadyOver => "Game over. Press 'New Game' to retry.".into(),
                };
            }
            Msg::NewGame => {
                self.game.start(&mut self.rng);
                self.status = format!("New game — watch carefully (length {}).", self.game.level());
            }
        }
    }
}

fn main() {
    assert_invariant!(SIMON_CONTRACT_HOLDS, check_simon_invariants().is_ok());
    let app = RelmApp::new("relm4.example.simon_says.game");
    app.run::<App>(());
}
