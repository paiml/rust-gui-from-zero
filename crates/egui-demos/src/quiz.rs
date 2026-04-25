//! Pure quiz state machine — no eframe, no I/O, no clocks.
//!
//! The view binaries layer eframe + a wall-clock timer on top of this.
//! Tests cover every state transition headlessly.

use contracts::ContractError;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, BufReader};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Question {
    pub text: String,
    pub choices: Vec<String>,
    pub correct_answer: usize,
}

impl Question {
    #[must_use]
    pub fn is_well_formed(&self) -> bool {
        !self.text.is_empty()
            && !self.choices.is_empty()
            && self.correct_answer < self.choices.len()
    }
}

#[derive(Debug, Clone)]
pub struct QuizGame {
    questions: Vec<Question>,
    current: usize,
    score: u32,
    game_over: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnswerOutcome {
    Correct,
    Incorrect,
    OutOfBounds,
    GameOver,
}

impl QuizGame {
    #[must_use]
    pub fn new(questions: Vec<Question>) -> Self {
        let game_over = questions.is_empty();
        Self {
            questions,
            current: 0,
            score: 0,
            game_over,
        }
    }

    #[must_use]
    pub fn questions(&self) -> &[Question] {
        &self.questions
    }

    #[must_use]
    pub fn current_index(&self) -> usize {
        self.current
    }

    #[must_use]
    pub fn current_question(&self) -> Option<&Question> {
        self.questions.get(self.current)
    }

    #[must_use]
    pub fn score(&self) -> u32 {
        self.score
    }

    #[must_use]
    pub fn total(&self) -> usize {
        self.questions.len()
    }

    #[must_use]
    pub fn is_game_over(&self) -> bool {
        self.game_over
    }

    /// Submit an answer for the current question.
    ///
    /// Returns the outcome and advances to the next question. If this was the
    /// last question, the game ends.
    pub fn answer(&mut self, choice: usize) -> AnswerOutcome {
        if self.game_over {
            return AnswerOutcome::GameOver;
        }
        // Invariant: while !game_over, current < questions.len() (see advance()).
        let question = &self.questions[self.current];
        if choice >= question.choices.len() {
            return AnswerOutcome::OutOfBounds;
        }
        let outcome = if choice == question.correct_answer {
            self.score = self.score.saturating_add(1);
            AnswerOutcome::Correct
        } else {
            AnswerOutcome::Incorrect
        };
        self.advance();
        outcome
    }

    /// Skip the current question without scoring (e.g. timeout).
    pub fn timeout(&mut self) {
        if !self.game_over {
            self.advance();
        }
    }

    fn advance(&mut self) {
        self.current = self.current.saturating_add(1);
        if self.current >= self.questions.len() {
            self.game_over = true;
        }
    }
}

/// Load a list of `Question` from a YAML file.
///
/// # Errors
///
/// Returns an [`io::Error`] if the file cannot be read or parsed.
pub fn load_questions(path: impl AsRef<Path>) -> io::Result<Vec<Question>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    serde_yaml::from_reader(reader).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// Per-step transition validator extracted for testability.
///
/// Negative-path tests can pass synthetic values (e.g. a score that decreased
/// or an `OutOfBounds` outcome) to cover every error branch without
/// hand-corrupting `QuizGame`'s internals.
///
/// # Errors
///
/// Returns [`contracts::ContractError`] if any per-step invariant is violated.
pub fn validate_step(
    before: u32,
    after: u32,
    last: u32,
    outcome: AnswerOutcome,
    step: usize,
    total: usize,
) -> Result<(), ContractError> {
    if after < last {
        return Err(ContractError {
            name: "QUIZ_SCORE_MONOTONIC",
            message: format!("score decreased from {last} to {after}"),
        });
    }
    if after > before.saturating_add(1) {
        return Err(ContractError {
            name: "QUIZ_SCORE_STEP_AT_MOST_ONE",
            message: format!("score jumped from {before} to {after}"),
        });
    }
    if matches!(
        outcome,
        AnswerOutcome::OutOfBounds | AnswerOutcome::GameOver
    ) {
        return Err(ContractError {
            name: "QUIZ_OUTCOME_VALID",
            message: "well-formed answer returned OutOfBounds or GameOver".into(),
        });
    }
    if step > total {
        return Err(ContractError {
            name: "QUIZ_TERMINATES",
            message: "loop ran longer than total question count".into(),
        });
    }
    Ok(())
}

/// Final-state validator: total score must not exceed question count.
///
/// # Errors
///
/// Returns [`contracts::ContractError`] if `score > total`.
pub fn validate_final(score: u32, total: usize) -> Result<(), ContractError> {
    if score as usize > total {
        return Err(ContractError {
            name: "QUIZ_SCORE_BOUND",
            message: format!("score {score} exceeds total {total}"),
        });
    }
    Ok(())
}

/// Provable contract: the score is monotonically non-decreasing across any
/// sequence of `answer` / `timeout` calls, and never exceeds the question
/// count. After the final question, `is_game_over()` is true.
///
/// # Errors
///
/// Returns [`contracts::ContractError`] if the invariant is violated.
pub fn check_quiz_invariants() -> Result<(), ContractError> {
    let questions = vec![
        Question {
            text: "1+1?".into(),
            choices: vec!["1".into(), "2".into(), "3".into()],
            correct_answer: 1,
        },
        Question {
            text: "Sky color?".into(),
            choices: vec!["green".into(), "blue".into()],
            correct_answer: 1,
        },
        Question {
            text: "Largest planet?".into(),
            choices: vec!["Earth".into(), "Mars".into(), "Jupiter".into()],
            correct_answer: 2,
        },
    ];
    let total = questions.len();
    let mut game = QuizGame::new(questions);

    let mut last_score = 0u32;
    let mut steps = 0usize;
    while !game.is_game_over() {
        let before = game.score();
        let outcome = game.answer(0);
        let after = game.score();
        steps += 1;
        validate_step(before, after, last_score, outcome, steps, total)?;
        last_score = after;
    }
    validate_final(game.score(), total)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    fn sample() -> Vec<Question> {
        vec![
            Question {
                text: "q1".into(),
                choices: vec!["a".into(), "b".into()],
                correct_answer: 1,
            },
            Question {
                text: "q2".into(),
                choices: vec!["a".into(), "b".into(), "c".into()],
                correct_answer: 2,
            },
        ]
    }

    #[test]
    fn empty_game_is_immediately_over() {
        let game = QuizGame::new(vec![]);
        assert!(game.is_game_over());
        assert_eq!(game.total(), 0);
        assert_eq!(game.score(), 0);
        assert!(game.current_question().is_none());
    }

    #[test]
    fn correct_answer_increments_score_and_advances() {
        let mut game = QuizGame::new(sample());
        assert_eq!(game.answer(1), AnswerOutcome::Correct);
        assert_eq!(game.score(), 1);
        assert_eq!(game.current_index(), 1);
        assert!(!game.is_game_over());
    }

    #[test]
    fn incorrect_answer_does_not_score_but_advances() {
        let mut game = QuizGame::new(sample());
        assert_eq!(game.answer(0), AnswerOutcome::Incorrect);
        assert_eq!(game.score(), 0);
        assert_eq!(game.current_index(), 1);
    }

    #[test]
    fn out_of_bounds_choice_does_not_advance() {
        let mut game = QuizGame::new(sample());
        assert_eq!(game.answer(99), AnswerOutcome::OutOfBounds);
        assert_eq!(game.score(), 0);
        assert_eq!(game.current_index(), 0);
    }

    #[test]
    fn timeout_advances_without_scoring() {
        let mut game = QuizGame::new(sample());
        game.timeout();
        assert_eq!(game.score(), 0);
        assert_eq!(game.current_index(), 1);
        game.timeout();
        assert!(game.is_game_over());
    }

    #[test]
    fn timeout_after_game_over_is_noop() {
        let mut game = QuizGame::new(sample());
        game.answer(1);
        game.answer(2);
        assert!(game.is_game_over());
        game.timeout();
        assert!(game.is_game_over());
    }

    #[test]
    fn answer_after_game_over_returns_game_over() {
        let mut game = QuizGame::new(sample());
        game.answer(1);
        game.answer(2);
        assert_eq!(game.answer(0), AnswerOutcome::GameOver);
    }

    #[test]
    fn finishing_all_questions_ends_game() {
        let mut game = QuizGame::new(sample());
        game.answer(1);
        assert_eq!(game.answer(2), AnswerOutcome::Correct);
        assert!(game.is_game_over());
        assert_eq!(game.score(), 2);
    }

    #[test]
    fn questions_accessor_returns_input() {
        let qs = sample();
        let game = QuizGame::new(qs.clone());
        assert_eq!(game.questions(), qs.as_slice());
    }

    #[test]
    fn current_question_tracks_index() {
        let mut game = QuizGame::new(sample());
        assert_eq!(game.current_question().unwrap().text, "q1");
        game.answer(0);
        assert_eq!(game.current_question().unwrap().text, "q2");
    }

    #[test]
    fn well_formed_question() {
        let q = Question {
            text: "ok?".into(),
            choices: vec!["x".into(), "y".into()],
            correct_answer: 0,
        };
        assert!(q.is_well_formed());
    }

    #[test]
    fn malformed_question_empty_text() {
        let q = Question {
            text: String::new(),
            choices: vec!["x".into()],
            correct_answer: 0,
        };
        assert!(!q.is_well_formed());
    }

    #[test]
    fn malformed_question_no_choices() {
        let q = Question {
            text: "ok?".into(),
            choices: vec![],
            correct_answer: 0,
        };
        assert!(!q.is_well_formed());
    }

    #[test]
    fn malformed_question_correct_oob() {
        let q = Question {
            text: "ok?".into(),
            choices: vec!["x".into()],
            correct_answer: 5,
        };
        assert!(!q.is_well_formed());
    }

    #[test]
    fn load_questions_round_trip() {
        let dir = std::env::temp_dir();
        let path = dir.join(format!("rgfz-quiz-{}.yaml", std::process::id()));
        let yaml = "- text: q1\n  choices: [a, b]\n  correct_answer: 1\n";
        std::fs::write(&path, yaml).unwrap();
        let qs = load_questions(&path).unwrap();
        let _ = std::fs::remove_file(&path);
        assert_eq!(qs.len(), 1);
        assert_eq!(qs[0].correct_answer, 1);
    }

    #[test]
    fn load_questions_missing_file() {
        let path = std::env::temp_dir().join("rgfz-quiz-missing-xyz.yaml");
        let _ = std::fs::remove_file(&path);
        assert!(load_questions(&path).is_err());
    }

    #[test]
    fn load_questions_bad_yaml() {
        let dir = std::env::temp_dir();
        let path = dir.join(format!("rgfz-quiz-bad-{}.yaml", std::process::id()));
        std::fs::write(&path, "not: [valid").unwrap();
        let result = load_questions(&path);
        let _ = std::fs::remove_file(&path);
        assert!(result.is_err());
    }

    #[test]
    fn invariant_holds_for_baseline() {
        check_quiz_invariants().unwrap();
    }

    #[test]
    fn answer_outcome_eq() {
        assert_eq!(AnswerOutcome::Correct, AnswerOutcome::Correct);
        assert_ne!(AnswerOutcome::Correct, AnswerOutcome::Incorrect);
    }

    #[test]
    fn validate_step_accepts_good_step() {
        validate_step(0, 1, 0, AnswerOutcome::Correct, 1, 3).unwrap();
        validate_step(2, 2, 2, AnswerOutcome::Incorrect, 2, 3).unwrap();
    }

    #[test]
    fn validate_step_rejects_score_decrease() {
        let err = validate_step(2, 1, 2, AnswerOutcome::Correct, 1, 3).unwrap_err();
        assert_eq!(err.name, "QUIZ_SCORE_MONOTONIC");
    }

    #[test]
    fn validate_step_rejects_score_jump() {
        let err = validate_step(0, 5, 0, AnswerOutcome::Correct, 1, 3).unwrap_err();
        assert_eq!(err.name, "QUIZ_SCORE_STEP_AT_MOST_ONE");
    }

    #[test]
    fn validate_step_rejects_out_of_bounds_outcome() {
        let err = validate_step(0, 0, 0, AnswerOutcome::OutOfBounds, 1, 3).unwrap_err();
        assert_eq!(err.name, "QUIZ_OUTCOME_VALID");
    }

    #[test]
    fn validate_step_rejects_game_over_outcome() {
        let err = validate_step(0, 0, 0, AnswerOutcome::GameOver, 1, 3).unwrap_err();
        assert_eq!(err.name, "QUIZ_OUTCOME_VALID");
    }

    #[test]
    fn validate_step_rejects_runaway_step() {
        let err = validate_step(0, 0, 0, AnswerOutcome::Incorrect, 99, 3).unwrap_err();
        assert_eq!(err.name, "QUIZ_TERMINATES");
    }

    #[test]
    fn validate_final_accepts_good_score() {
        validate_final(2, 3).unwrap();
        validate_final(3, 3).unwrap();
    }

    #[test]
    fn validate_final_rejects_overflow() {
        let err = validate_final(99, 3).unwrap_err();
        assert_eq!(err.name, "QUIZ_SCORE_BOUND");
    }
}
