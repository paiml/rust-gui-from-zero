//! `fltk-pong-step3-speed` — pong with a speed slider.
//!
//! **Provable contract:** `PONG_BALL_IN_BOUNDS` — physics keeps the ball
//! and paddle inside the playfield over 1000 ticks.

#![allow(clippy::wildcard_imports, clippy::too_many_lines)]
use contracts::assert_invariant;
use fltk::{enums::*, prelude::*, *};
use fltk_demos::pong::{check_pong_invariants, Pong};
use std::cell::RefCell;
use std::rc::Rc;

const KEY_A: Key = Key::from_char('a');
const KEY_D: Key = Key::from_char('d');

fn main() {
    assert_invariant!(PONG_CONTRACT_HOLDS, check_pong_invariants().is_ok());

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
        move |b| {
            let running = game.borrow_mut().toggle_running();
            if running {
                b.set_label("Stop");
                b.set_color(Color::Red);
            } else {
                b.set_label("Start");
                b.set_color(Color::Green);
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
        move |_| {
            let mut g = game.borrow_mut();
            let _ = g.step();
            ball.resize(
                g.ball.pos.0,
                g.ball.pos.1,
                g.bounds.ball_diameter,
                g.bounds.ball_diameter,
            );
            drop(g);
            wind_for_idle.redraw();
            app::sleep(0.016);
        }
    });

    if let Err(e) = app.run() {
        eprintln!("fltk-pong-step3 exited: {e}");
    }
}
