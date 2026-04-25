//! Pure pong physics for the `fltk-pong*` family of demos.
//!
//! The canonical FLTK demos mix UI plumbing (FLTK frames, callbacks) with
//! physics (ball position, paddle clamping, score). This module owns the
//! physics half so the simulation can be exercised without an FLTK display.

#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Direction {
    Positive = 1,
    Negative = -1,
}

impl Direction {
    #[must_use]
    pub fn flip(self) -> Self {
        match self {
            Self::Positive => Self::Negative,
            Self::Negative => Self::Positive,
        }
    }

    #[must_use]
    pub fn as_i32(self) -> i32 {
        self as i32
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Bounds {
    pub width: i32,
    pub height: i32,
    pub paddle_y: i32,
    pub paddle_w: i32,
    pub paddle_h: i32,
    pub ball_diameter: i32,
}

impl Default for Bounds {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            paddle_y: 540,
            paddle_w: 160,
            paddle_h: 20,
            ball_diameter: 40,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BallState {
    pub pos: (i32, i32),
    pub dir: (Direction, Direction),
    pub has_moved: bool,
}

impl BallState {
    #[must_use]
    pub fn new(start: (i32, i32)) -> Self {
        Self {
            pos: start,
            dir: (Direction::Positive, Direction::Positive),
            has_moved: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Pong {
    pub bounds: Bounds,
    pub ball: BallState,
    pub paddle_pos: i32,
    pub game_running: bool,
    pub game_speed: f64,
    pub score: u32,
}

impl Default for Pong {
    fn default() -> Self {
        let bounds = Bounds::default();
        let start = (bounds.width / 2, bounds.height / 2);
        Self {
            bounds,
            ball: BallState::new(start),
            paddle_pos: (bounds.width - bounds.paddle_w) / 2,
            game_running: false,
            game_speed: 5.0,
            score: 0,
        }
    }
}

impl Pong {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn paddle_max(&self) -> i32 {
        self.bounds.width - self.bounds.paddle_w
    }

    pub fn move_paddle_left(&mut self, by: i32) {
        self.paddle_pos = (self.paddle_pos - by).max(0);
    }

    pub fn move_paddle_right(&mut self, by: i32) {
        self.paddle_pos = (self.paddle_pos + by).min(self.paddle_max());
    }

    pub fn set_paddle(&mut self, x: i32) {
        self.paddle_pos = x.clamp(0, self.paddle_max());
    }

    pub fn start(&mut self) {
        self.game_running = true;
    }

    pub fn stop(&mut self) {
        self.game_running = false;
    }

    pub fn toggle_running(&mut self) -> bool {
        self.game_running = !self.game_running;
        self.game_running
    }

    pub fn set_speed(&mut self, speed: f64) {
        self.game_speed = speed.clamp(1.0, 10.0);
    }

    pub fn reset_score(&mut self) {
        self.score = 0;
    }

    /// Advance one tick of pong physics. Returns `true` if the ball hit
    /// the paddle on this tick (callers use this to update the score
    /// display in v4).
    #[allow(clippy::cast_possible_truncation)]
    pub fn step(&mut self) -> bool {
        if !self.game_running {
            return false;
        }
        let factor = self.game_speed.clamp(1.0, 10.0) / 5.0;
        let dx = (5.0 * factor * f64::from(self.ball.dir.0.as_i32())) as i32;
        let dy = (5.0 * factor * f64::from(self.ball.dir.1.as_i32())) as i32;
        self.ball.pos.0 += dx;
        self.ball.pos.1 += dy;
        self.ball.has_moved = true;

        let mut paddle_hit = false;
        let paddle_y_top = self.bounds.paddle_y - self.bounds.ball_diameter;
        if self.ball.pos.1 >= paddle_y_top
            && self.ball.pos.0 > self.paddle_pos - self.bounds.ball_diameter
            && self.ball.pos.0 < self.paddle_pos + self.bounds.paddle_w
        {
            self.ball.dir.1 = Direction::Negative;
            self.score += 100;
            paddle_hit = true;
        }
        if self.ball.pos.1 <= 0 {
            self.ball.dir.1 = Direction::Positive;
        }
        if self.ball.pos.0 >= self.bounds.width - self.bounds.ball_diameter {
            self.ball.dir.0 = Direction::Negative;
        }
        if self.ball.pos.0 <= 0 {
            self.ball.dir.0 = Direction::Positive;
        }

        let max_x = self.bounds.width - self.bounds.ball_diameter;
        let max_y = self.bounds.height - self.bounds.ball_diameter;
        self.ball.pos.0 = self.ball.pos.0.clamp(0, max_x);
        self.ball.pos.1 = self.ball.pos.1.clamp(0, max_y);

        paddle_hit
    }
}

/// Validate a single snapshot of pong state — returns `Err` describing
/// the first violation encountered. Public so the contract harness can
/// drive negative tests with synthetic bad state.
///
/// # Errors
///
/// Returns [`contracts::ContractError`] if the ball or paddle is out of
/// bounds at `tick`.
pub fn validate_state(p: &Pong, tick: usize) -> Result<(), contracts::ContractError> {
    let max_x = p.bounds.width - p.bounds.ball_diameter;
    let max_y = p.bounds.height - p.bounds.ball_diameter;
    let paddle_max = p.paddle_max();
    if p.ball.pos.0 < 0 || p.ball.pos.0 > max_x || p.ball.pos.1 < 0 || p.ball.pos.1 > max_y {
        return Err(contracts::ContractError {
            name: "PONG_BALL_IN_BOUNDS",
            message: format!("ball escaped bounds at tick {tick}"),
        });
    }
    if p.paddle_pos < 0 || p.paddle_pos > paddle_max {
        return Err(contracts::ContractError {
            name: "PONG_BALL_IN_BOUNDS",
            message: format!("paddle escaped bounds at tick {tick}"),
        });
    }
    Ok(())
}

/// Provable contract: after an arbitrary number of physics ticks the ball
/// stays inside the playfield AND the paddle stays inside the screen.
///
/// # Errors
///
/// Returns [`contracts::ContractError`] when any tick produces an
/// out-of-bounds ball or paddle.
pub fn check_pong_invariants() -> Result<(), contracts::ContractError> {
    let mut p = Pong::new();
    p.start();
    p.set_speed(10.0);
    for tick in 0..1000 {
        let _ = p.step();
        validate_state(&p, tick)?;
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn direction_flip_inverts() {
        assert_eq!(Direction::Positive.flip(), Direction::Negative);
        assert_eq!(Direction::Negative.flip(), Direction::Positive);
    }

    #[test]
    fn direction_as_i32_matches_repr() {
        assert_eq!(Direction::Positive.as_i32(), 1);
        assert_eq!(Direction::Negative.as_i32(), -1);
    }

    #[test]
    fn new_pong_centered() {
        let p = Pong::new();
        assert_eq!(p.ball.pos, (400, 300));
        assert!(!p.game_running);
        assert_eq!(p.score, 0);
    }

    #[test]
    fn move_paddle_left_clamps_at_zero() {
        let mut p = Pong::new();
        p.paddle_pos = 0;
        p.move_paddle_left(30);
        assert_eq!(p.paddle_pos, 0);
    }

    #[test]
    fn move_paddle_right_clamps_at_max() {
        let mut p = Pong::new();
        p.paddle_pos = p.paddle_max();
        p.move_paddle_right(30);
        assert_eq!(p.paddle_pos, p.paddle_max());
    }

    #[test]
    fn move_paddle_left_decrements() {
        let mut p = Pong::new();
        p.paddle_pos = 100;
        p.move_paddle_left(30);
        assert_eq!(p.paddle_pos, 70);
    }

    #[test]
    fn move_paddle_right_increments() {
        let mut p = Pong::new();
        p.paddle_pos = 100;
        p.move_paddle_right(30);
        assert_eq!(p.paddle_pos, 130);
    }

    #[test]
    fn set_paddle_clamps_below_zero() {
        let mut p = Pong::new();
        p.set_paddle(-100);
        assert_eq!(p.paddle_pos, 0);
    }

    #[test]
    fn set_paddle_clamps_above_max() {
        let mut p = Pong::new();
        p.set_paddle(99_999);
        assert_eq!(p.paddle_pos, p.paddle_max());
    }

    #[test]
    fn set_paddle_in_range() {
        let mut p = Pong::new();
        p.set_paddle(123);
        assert_eq!(p.paddle_pos, 123);
    }

    #[test]
    fn step_does_nothing_when_stopped() {
        let mut p = Pong::new();
        let before = p.ball.pos;
        let hit = p.step();
        assert!(!hit);
        assert_eq!(p.ball.pos, before);
    }

    #[test]
    fn start_then_step_moves_ball() {
        let mut p = Pong::new();
        p.start();
        let before = p.ball.pos;
        p.step();
        assert_ne!(p.ball.pos, before);
        assert!(p.ball.has_moved);
    }

    #[test]
    fn toggle_running_flips() {
        let mut p = Pong::new();
        assert!(p.toggle_running());
        assert!(p.game_running);
        assert!(!p.toggle_running());
        assert!(!p.game_running);
    }

    #[test]
    fn stop_after_start() {
        let mut p = Pong::new();
        p.start();
        p.stop();
        assert!(!p.game_running);
    }

    #[test]
    fn set_speed_clamps_low() {
        let mut p = Pong::new();
        p.set_speed(0.0);
        assert!((p.game_speed - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn set_speed_clamps_high() {
        let mut p = Pong::new();
        p.set_speed(99.0);
        assert!((p.game_speed - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn set_speed_in_range() {
        let mut p = Pong::new();
        p.set_speed(7.5);
        assert!((p.game_speed - 7.5).abs() < f64::EPSILON);
    }

    #[test]
    fn reset_score_zeroes() {
        let mut p = Pong::new();
        p.score = 999;
        p.reset_score();
        assert_eq!(p.score, 0);
    }

    #[test]
    fn ball_bounces_off_top_wall() {
        let mut p = Pong::new();
        p.start();
        p.ball.pos = (400, 0);
        p.ball.dir.1 = Direction::Negative;
        p.step();
        assert_eq!(p.ball.dir.1, Direction::Positive);
    }

    #[test]
    fn ball_bounces_off_left_wall() {
        let mut p = Pong::new();
        p.start();
        p.ball.pos = (0, 300);
        p.ball.dir.0 = Direction::Negative;
        p.step();
        assert_eq!(p.ball.dir.0, Direction::Positive);
    }

    #[test]
    fn ball_bounces_off_right_wall() {
        let mut p = Pong::new();
        p.start();
        let max_x = p.bounds.width - p.bounds.ball_diameter;
        p.ball.pos = (max_x, 300);
        p.ball.dir.0 = Direction::Positive;
        p.step();
        assert_eq!(p.ball.dir.0, Direction::Negative);
    }

    #[test]
    fn paddle_hit_increments_score_and_bounces() {
        let mut p = Pong::new();
        p.start();
        // Place ball directly at the paddle's vertical zone, x within
        // paddle's horizontal coverage.
        p.ball.pos = (
            p.paddle_pos + 20,
            p.bounds.paddle_y - p.bounds.ball_diameter,
        );
        p.ball.dir.1 = Direction::Positive;
        let hit = p.step();
        assert!(hit);
        assert_eq!(p.score, 100);
        assert_eq!(p.ball.dir.1, Direction::Negative);
    }

    #[test]
    fn provable_contract_holds() {
        check_pong_invariants().expect("contract should hold");
    }

    #[test]
    fn validate_state_accepts_good_state() {
        let p = Pong::new();
        validate_state(&p, 0).expect("default state should be valid");
    }

    #[test]
    fn validate_state_rejects_oob_ball_x() {
        let mut p = Pong::new();
        p.ball.pos.0 = -10;
        let err = validate_state(&p, 7).expect_err("expected ball oob error");
        assert_eq!(err.name, "PONG_BALL_IN_BOUNDS");
        assert!(err.message.contains("ball"));
        assert!(err.message.contains("tick 7"));
    }

    #[test]
    fn validate_state_rejects_oob_ball_y() {
        let mut p = Pong::new();
        p.ball.pos.1 = p.bounds.height + 100;
        let err = validate_state(&p, 9).expect_err("expected ball oob error");
        assert_eq!(err.name, "PONG_BALL_IN_BOUNDS");
        assert!(err.message.contains("ball"));
    }

    #[test]
    fn validate_state_rejects_oob_paddle() {
        let mut p = Pong::new();
        p.paddle_pos = -1;
        let err = validate_state(&p, 3).expect_err("expected paddle oob error");
        assert_eq!(err.name, "PONG_BALL_IN_BOUNDS");
        assert!(err.message.contains("paddle"));
        assert!(err.message.contains("tick 3"));
    }

    #[test]
    fn validate_state_rejects_oob_paddle_high() {
        let mut p = Pong::new();
        p.paddle_pos = p.paddle_max() + 1;
        let err = validate_state(&p, 4).expect_err("expected paddle oob error");
        assert!(err.message.contains("paddle"));
    }
}
