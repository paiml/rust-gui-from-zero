//! `egui-quiz` — single-file quiz player.
//!
//! **Provable contract:** `QUIZ_SCORE_MONOTONIC` and `QUIZ_SCORE_BOUND` —
//! score never decreases per step and never exceeds the question count.
//! Verified at startup against a baseline 3-question quiz.

#![allow(clippy::wildcard_imports, clippy::too_many_lines)]
use contracts::assert_invariant;
use eframe::egui;
use egui_demos::quiz::{check_quiz_invariants, load_questions, AnswerOutcome, Question, QuizGame};
use std::time::{Duration, Instant};

const QUESTIONS_FILE: &str = "questions.yaml";
const TIME_PER_QUESTION_SECS: u64 = 15;

struct QuizApp {
    game: QuizGame,
    timer: Instant,
    time_per_question: Duration,
}

impl QuizApp {
    fn new(questions: Vec<Question>) -> Self {
        Self {
            game: QuizGame::new(questions),
            timer: Instant::now(),
            time_per_question: Duration::from_secs(TIME_PER_QUESTION_SECS),
        }
    }

    fn reset(&mut self, questions: Vec<Question>) {
        self.game = QuizGame::new(questions);
        self.timer = Instant::now();
    }
}

impl eframe::App for QuizApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.game.is_game_over() {
                ui.heading("Game Over!");
                ui.label(format!(
                    "Final Score: {}/{}",
                    self.game.score(),
                    self.game.total()
                ));
                if ui.button("Play Again").clicked() {
                    let questions = load_questions(QUESTIONS_FILE).unwrap_or_default();
                    self.reset(questions);
                }
                return;
            }
            let total = self.game.total();
            let current = self.game.current_index();
            let Some(q) = self.game.current_question().cloned() else {
                return;
            };
            ui.heading(format!("Question {}/{total}", current + 1));
            ui.label(&q.text);
            let elapsed = self.timer.elapsed();
            let time_left = self.time_per_question.saturating_sub(elapsed).as_secs();
            ui.label(format!("Time left: {time_left}s"));
            let mut answered = false;
            for (i, choice) in q.choices.iter().enumerate() {
                if ui.button(choice).clicked() {
                    let _ = self.game.answer(i);
                    answered = true;
                }
            }
            if !answered && elapsed >= self.time_per_question {
                self.game.timeout();
                self.timer = Instant::now();
            } else if answered {
                self.timer = Instant::now();
            }
            let _ = AnswerOutcome::Correct;
        });
        ctx.request_repaint();
    }
}

fn main() -> eframe::Result<()> {
    assert_invariant!(QUIZ_CONTRACT_HOLDS, check_quiz_invariants().is_ok());
    let questions = load_questions(QUESTIONS_FILE).unwrap_or_default();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Quiz",
        native_options,
        Box::new(|_cc| Box::new(QuizApp::new(questions))),
    )
}
