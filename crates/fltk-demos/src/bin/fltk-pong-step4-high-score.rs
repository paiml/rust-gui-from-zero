//! `fltk-pong-step4-high-score` — pong with score + persistent high score.
//!
//! **Provable contracts:** `PONG_BALL_IN_BOUNDS` and
//! `FLTK_HIGHSCORE_ROUNDTRIP`, both verified at startup.

#![allow(clippy::wildcard_imports, clippy::too_many_lines)]
use contracts::assert_invariant;
use fltk::{enums::*, prelude::*, *};
use fltk_demos::highscore::{check_highscore_roundtrip, read_high_score, write_high_score};
use fltk_demos::pong::{check_pong_invariants, Pong};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

const KEY_A: Key = Key::from_char('a');
const KEY_D: Key = Key::from_char('d');
const HIGH_SCORE_FILE: &str = "highscore.txt";

fn main() {
    assert_invariant!(PONG_CONTRACT_HOLDS, check_pong_invariants().is_ok());
    let tmp = std::env::temp_dir().join(format!("rgfz-fltk-hs-startup-{}.txt", std::process::id()));
    assert_invariant!(
        FLTK_HIGHSCORE_CONTRACT_HOLDS,
        check_highscore_roundtrip(&tmp).is_ok()
    );
    let _ = std::fs::remove_file(&tmp);

    let high_score_path: PathBuf = PathBuf::from(HIGH_SCORE_FILE);

    let app = app::App::default();
    let mut wind = window::Window::default()
        .with_size(800, 600)
        .center_screen()
        .with_label("Pong!");
    let mut ball = frame::Frame::new(0, 0, 40, 40, None);
    ball.set_frame(FrameType::OFlatBox);
    ball.set_color(Color::White);
    wind.set_color(Color::Black);

    let mut btn = button::Button::new(10, 10, 100, 40, "Start");
    btn.set_color(Color::Green);

    let mut speed_slider = valuator::HorNiceSlider::new(120, 10, 200, 40, "Speed");
    speed_slider.set_range(1.0, 10.0);
    speed_slider.set_value(5.0);
    speed_slider.set_step(1.0, 1);
    speed_slider.set_color(Color::Blue);

    let mut score_display = frame::Frame::new(330, 10, 200, 40, "Score: 0");
    score_display.set_label_size(20);
    score_display.set_label_color(Color::White);

    let initial_high = read_high_score(&high_score_path).unwrap_or(0);
    let mut high_score_display = frame::Frame::new(
        540,
        10,
        250,
        40,
        format!("High Score: {initial_high}").as_str(),
    );
    high_score_display.set_label_size(20);
    high_score_display.set_label_color(Color::Yellow);

    wind.end();
    wind.show();

    let game = Rc::new(RefCell::new(Pong::new()));

    wind.draw({
        let game = game.clone();
        move |_| {
            let g = game.borrow();
            draw::set_draw_color(Color::White);
            draw::draw_rectf(
                g.paddle_pos,
                g.bounds.paddle_y,
                g.bounds.paddle_w,
                g.bounds.paddle_h,
            );
        }
    });

    wind.handle({
        let game = game.clone();
        move |_, ev| match ev {
            Event::Focus => true,
            Event::KeyDown => {
                let mut g = game.borrow_mut();
                match app::event_key() {
                    Key::Left | KEY_A => g.move_paddle_left(30),
                    Key::Right | KEY_D => g.move_paddle_right(30),
                    _ => return false,
                }
                true
            }
            Event::Move => {
                let mut g = game.borrow_mut();
                g.set_paddle(app::event_coords().0);
                true
            }
            _ => false,
        }
    });

    btn.set_callback({
        let game = game.clone();
        let mut score_display = score_display.clone();
        let mut high_score_display = high_score_display.clone();
        let path = high_score_path.clone();
        move |b| {
            let running = game.borrow_mut().toggle_running();
            if running {
                b.set_label("Stop");
                b.set_color(Color::Red);
            } else {
                b.set_label("Start");
                b.set_color(Color::Green);
                let current = game.borrow().score;
                let high = read_high_score(&path).unwrap_or(0);
                if current > high {
                    if let Err(e) = write_high_score(&path, current) {
                        eprintln!("failed to persist high score: {e}");
                    }
                    high_score_display.set_label(format!("High Score: {current}").as_str());
                }
                game.borrow_mut().reset_score();
                score_display.set_label("Score: 0");
            }
            b.redraw();
        }
    });

    speed_slider.set_callback({
        let game = game.clone();
        move |s| game.borrow_mut().set_speed(s.value())
    });

    let mut wind_for_idle = wind.clone();
    app::add_idle3({
        let game = game.clone();
        let mut score_display = score_display.clone();
        move |_| {
            let mut g = game.borrow_mut();
            let hit = g.step();
            ball.resize(
                g.ball.pos.0,
                g.ball.pos.1,
                g.bounds.ball_diameter,
                g.bounds.ball_diameter,
            );
            if hit {
                score_display.set_label(format!("Score: {}", g.score).as_str());
            }
            drop(g);
            wind_for_idle.redraw();
            app::sleep(0.016);
        }
    });

    if let Err(e) = app.run() {
        eprintln!("fltk-pong-step4 exited: {e}");
    }
}
