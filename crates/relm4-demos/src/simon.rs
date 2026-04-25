//! Pure Simon-Says state machine — no GUI, no system clock.
//!
//! View binaries inject `rand::thread_rng()` at runtime; tests inject a
//! seeded `StdRng` so the assertions are deterministic.

use contracts::ContractError;
use rand::Rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Red,
    Green,
    Blue,
    Yellow,
}

impl Color {
    #[must_use]
    pub fn from_index(i: u32) -> Self {
        match i % 4 {
            0 => Self::Red,
            1 => Self::Green,
            2 => Self::Blue,
            _ => Self::Yellow,
        }
    }

    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Red => "Red",
            Self::Green => "Green",
            Self::Blue => "Blue",
            Self::Yellow => "Yellow",
        }
    }
}

/// Sample a uniformly random `Color` using the supplied RNG.
pub fn random_color<R: Rng>(rng: &mut R) -> Color {
    Color::from_index(rng.gen_range(0..4))
}

/// Return any color that is not `c` — handy for synthesising a "wrong"
/// press in deterministic tests and contract checks.
#[must_use]
pub fn other_color(c: Color) -> Color {
    match c {
        Color::Red => Color::Green,
        Color::Green | Color::Blue | Color::Yellow => Color::Red,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PressOutcome {
    /// Press arrived before the game was started — ignored.
    Idle,
    /// Press arrived after the game ended — ignored.
    AlreadyOver,
    /// Press matched the next color but the round is still going.
    Correct,
    /// Press matched the final color of the round; sequence extended.
    RoundComplete,
    /// Press did not match; game is now over.
    Wrong,
}

#[derive(Debug, Clone, Default)]
pub struct SimonGame {
    pub sequence: Vec<Color>,
    pub current_index: usize,
    pub game_over: bool,
}

impl SimonGame {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Start (or restart) the game with a single random color.
    pub fn start<R: Rng>(&mut self, rng: &mut R) {
        self.sequence.clear();
        self.sequence.push(random_color(rng));
        self.current_index = 0;
        self.game_over = false;
    }

    /// Process a player press and return the resulting outcome.
    pub fn press<R: Rng>(&mut self, color: Color, rng: &mut R) -> PressOutcome {
        if self.sequence.is_empty() {
            return PressOutcome::Idle;
        }
        if self.game_over {
            return PressOutcome::AlreadyOver;
        }
        if color != self.sequence[self.current_index] {
            self.game_over = true;
            return PressOutcome::Wrong;
        }
        self.current_index += 1;
        if self.current_index == self.sequence.len() {
            self.sequence.push(random_color(rng));
            self.current_index = 0;
            return PressOutcome::RoundComplete;
        }
        PressOutcome::Correct
    }

    #[must_use]
    pub fn level(&self) -> usize {
        self.sequence.len()
    }

    #[must_use]
    pub fn is_game_over(&self) -> bool {
        self.game_over
    }
}

/// Validate a single (sequence-length, index, game-over) snapshot —
/// extracted so negative-path tests can exercise every error branch
/// with synthetic data.
///
/// # Errors
///
/// Returns [`contracts::ContractError`] when an invariant is violated.
pub fn validate_state(
    seq_len: usize,
    current_index: usize,
    game_over: bool,
) -> Result<(), ContractError> {
    if seq_len == 0 && current_index != 0 {
        return Err(ContractError {
            name: "SIMON_INDEX_BOUNDS",
            message: format!("empty sequence but current_index={current_index}"),
        });
    }
    if seq_len > 0 && current_index >= seq_len && !game_over {
        return Err(ContractError {
            name: "SIMON_INDEX_BOUNDS",
            message: format!(
                "current_index {current_index} not < seq_len {seq_len} (and not game_over)"
            ),
        });
    }
    Ok(())
}

/// Validate the `outcome` of a press during a deterministic
/// correct-press walk — extracted so negative-path tests can pass
/// synthetic outcomes.
///
/// # Errors
///
/// Returns [`contracts::ContractError`] if `outcome` indicates the
/// walk diverged (e.g. `Wrong` or `AlreadyOver`).
pub fn validate_walk_step(outcome: PressOutcome) -> Result<(), ContractError> {
    if matches!(outcome, PressOutcome::Wrong | PressOutcome::AlreadyOver) {
        return Err(ContractError {
            name: "SIMON_DETERMINISTIC_WALK",
            message: format!("unexpected outcome {outcome:?} on a correct press"),
        });
    }
    Ok(())
}

/// Validate the terminal state after a deliberately wrong press.
///
/// # Errors
///
/// Returns [`contracts::ContractError`] if the outcome is not
/// [`PressOutcome::Wrong`] or the game did not flag itself over.
pub fn validate_terminal_wrong(
    outcome: PressOutcome,
    game_over: bool,
) -> Result<(), ContractError> {
    if outcome != PressOutcome::Wrong || !game_over {
        return Err(ContractError {
            name: "SIMON_GAME_OVER",
            message: format!("wrong press did not end the game (outcome={outcome:?})"),
        });
    }
    Ok(())
}

/// Validate the response to a press issued after the game has ended.
///
/// # Errors
///
/// Returns [`contracts::ContractError`] unless `outcome` is
/// [`PressOutcome::AlreadyOver`].
pub fn validate_already_over(outcome: PressOutcome) -> Result<(), ContractError> {
    if outcome != PressOutcome::AlreadyOver {
        return Err(ContractError {
            name: "SIMON_GAME_OVER_FROZEN",
            message: format!("press after game over returned {outcome:?}"),
        });
    }
    Ok(())
}

/// Provable contract: across a deterministic Simon-Says walk the
/// `current_index` is always `< sequence.len()` (unless the game has
/// just ended) and `Color::from_index` is total over `0..4`.
///
/// # Errors
///
/// Returns [`contracts::ContractError`] if any invariant fails.
pub fn check_simon_invariants() -> Result<(), ContractError> {
    use rand::SeedableRng;
    let mut rng = rand::rngs::StdRng::seed_from_u64(0x0011_5EE5);
    let mut game = SimonGame::new();
    validate_state(game.level(), game.current_index, game.is_game_over())?;
    game.start(&mut rng);
    validate_state(game.level(), game.current_index, game.is_game_over())?;
    for _ in 0..16 {
        let expected = game.sequence[game.current_index];
        let outcome = game.press(expected, &mut rng);
        validate_state(game.level(), game.current_index, game.is_game_over())?;
        validate_walk_step(outcome)?;
    }
    let wrong = other_color(game.sequence[game.current_index]);
    let outcome = game.press(wrong, &mut rng);
    validate_terminal_wrong(outcome, game.is_game_over())?;
    let outcome = game.press(Color::Red, &mut rng);
    validate_already_over(outcome)?;
    for i in 0..4 {
        let _color = Color::from_index(i);
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    fn rng() -> StdRng {
        StdRng::seed_from_u64(7)
    }

    #[test]
    fn color_from_index_covers_all_four() {
        assert_eq!(Color::from_index(0), Color::Red);
        assert_eq!(Color::from_index(1), Color::Green);
        assert_eq!(Color::from_index(2), Color::Blue);
        assert_eq!(Color::from_index(3), Color::Yellow);
        assert_eq!(Color::from_index(4), Color::Red);
        assert_eq!(Color::from_index(99), Color::Yellow);
    }

    #[test]
    fn color_labels() {
        assert_eq!(Color::Red.label(), "Red");
        assert_eq!(Color::Green.label(), "Green");
        assert_eq!(Color::Blue.label(), "Blue");
        assert_eq!(Color::Yellow.label(), "Yellow");
    }

    #[test]
    fn new_game_is_idle() {
        let game = SimonGame::new();
        assert_eq!(game.level(), 0);
        assert!(!game.is_game_over());
    }

    #[test]
    fn press_before_start_is_idle() {
        let mut game = SimonGame::new();
        let mut r = rng();
        assert_eq!(game.press(Color::Red, &mut r), PressOutcome::Idle);
    }

    #[test]
    fn start_seeds_one_color() {
        let mut game = SimonGame::new();
        let mut r = rng();
        game.start(&mut r);
        assert_eq!(game.level(), 1);
        assert_eq!(game.current_index, 0);
        assert!(!game.is_game_over());
    }

    #[test]
    fn correct_press_extends_sequence() {
        let mut game = SimonGame::new();
        let mut r = rng();
        game.start(&mut r);
        let expected = game.sequence[0];
        let outcome = game.press(expected, &mut r);
        assert_eq!(outcome, PressOutcome::RoundComplete);
        assert_eq!(game.level(), 2);
        assert_eq!(game.current_index, 0);
    }

    #[test]
    fn wrong_press_ends_game() {
        let mut game = SimonGame::new();
        let mut r = rng();
        game.start(&mut r);
        let wrong = other_color(game.sequence[0]);
        let outcome = game.press(wrong, &mut r);
        assert_eq!(outcome, PressOutcome::Wrong);
        assert!(game.is_game_over());
    }

    #[test]
    fn press_after_game_over_is_already_over() {
        let mut game = SimonGame::new();
        let mut r = rng();
        game.start(&mut r);
        let wrong = other_color(game.sequence[0]);
        game.press(wrong, &mut r);
        assert_eq!(game.press(Color::Red, &mut r), PressOutcome::AlreadyOver);
    }

    #[test]
    fn correct_partial_press_returns_correct() {
        let mut game = SimonGame::new();
        let mut r = StdRng::seed_from_u64(123);
        game.start(&mut r);
        // Force a 2-element sequence by completing one round.
        let first = game.sequence[0];
        game.press(first, &mut r);
        assert_eq!(game.level(), 2);
        let outcome = game.press(game.sequence[0], &mut r);
        assert_eq!(outcome, PressOutcome::Correct);
        assert_eq!(game.current_index, 1);
    }

    #[test]
    fn restart_clears_state() {
        let mut game = SimonGame::new();
        let mut r = rng();
        game.start(&mut r);
        let wrong = other_color(game.sequence[0]);
        game.press(wrong, &mut r);
        assert!(game.is_game_over());
        game.start(&mut r);
        assert!(!game.is_game_over());
        assert_eq!(game.level(), 1);
        assert_eq!(game.current_index, 0);
    }

    #[test]
    fn random_color_is_one_of_four() {
        let mut r = rng();
        for _ in 0..32 {
            let _ = random_color(&mut r);
        }
    }

    #[test]
    fn validate_state_accepts_idle() {
        validate_state(0, 0, false).unwrap();
    }

    #[test]
    fn validate_state_accepts_mid_round() {
        validate_state(3, 1, false).unwrap();
    }

    #[test]
    fn validate_state_accepts_game_over_at_end() {
        validate_state(3, 3, true).unwrap();
    }

    #[test]
    fn validate_state_rejects_index_in_empty_sequence() {
        let err = validate_state(0, 1, false).unwrap_err();
        assert_eq!(err.name, "SIMON_INDEX_BOUNDS");
    }

    #[test]
    fn validate_state_rejects_index_past_seq_without_game_over() {
        let err = validate_state(3, 3, false).unwrap_err();
        assert_eq!(err.name, "SIMON_INDEX_BOUNDS");
    }

    #[test]
    fn invariants_hold() {
        check_simon_invariants().unwrap();
    }

    #[test]
    fn other_color_is_always_different() {
        for i in 0..4 {
            let c = Color::from_index(i);
            assert_ne!(other_color(c), c);
        }
    }

    #[test]
    fn validate_walk_step_accepts_correct() {
        validate_walk_step(PressOutcome::Correct).unwrap();
        validate_walk_step(PressOutcome::RoundComplete).unwrap();
        validate_walk_step(PressOutcome::Idle).unwrap();
    }

    #[test]
    fn validate_walk_step_rejects_wrong() {
        let err = validate_walk_step(PressOutcome::Wrong).unwrap_err();
        assert_eq!(err.name, "SIMON_DETERMINISTIC_WALK");
    }

    #[test]
    fn validate_walk_step_rejects_already_over() {
        let err = validate_walk_step(PressOutcome::AlreadyOver).unwrap_err();
        assert_eq!(err.name, "SIMON_DETERMINISTIC_WALK");
    }

    #[test]
    fn validate_terminal_wrong_accepts_wrong_with_game_over() {
        validate_terminal_wrong(PressOutcome::Wrong, true).unwrap();
    }

    #[test]
    fn validate_terminal_wrong_rejects_wrong_without_game_over() {
        let err = validate_terminal_wrong(PressOutcome::Wrong, false).unwrap_err();
        assert_eq!(err.name, "SIMON_GAME_OVER");
    }

    #[test]
    fn validate_terminal_wrong_rejects_non_wrong_outcome() {
        let err = validate_terminal_wrong(PressOutcome::Correct, true).unwrap_err();
        assert_eq!(err.name, "SIMON_GAME_OVER");
    }

    #[test]
    fn validate_already_over_accepts_already_over() {
        validate_already_over(PressOutcome::AlreadyOver).unwrap();
    }

    #[test]
    fn validate_already_over_rejects_other_outcomes() {
        for outcome in [
            PressOutcome::Correct,
            PressOutcome::RoundComplete,
            PressOutcome::Wrong,
            PressOutcome::Idle,
        ] {
            let err = validate_already_over(outcome).unwrap_err();
            assert_eq!(err.name, "SIMON_GAME_OVER_FROZEN");
        }
    }
}
