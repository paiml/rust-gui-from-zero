//! `egui-quiz-multi` — quiz player that loads from a CLI-supplied YAML file.
//!
//! **Provable contract:** `QUIZ_SCORE_MONOTONIC` and `QUIZ_SCORE_BOUND` —
//! verified at startup against the same baseline as `egui-quiz`.

#![allow(clippy::wildcard_imports, clippy::too_many_lines)]
use contracts::assert_invariant;
use eframe::egui;
use egui_demos::quiz::{check_quiz_invariants, load_questions, Question, QuizGame};
use std::env;
use std::time::{Duration, Instant};

const DEFAULT_QUIZ: &str = "default_quiz.yaml";
const TIME_PER_QUESTION_SECS: u64 = 15;

struct QuizApp {
    game: QuizGame,
    timer: Instant,
    time_per_question: Duration,
    quiz_file: String,
}

impl QuizApp {
    fn new(questions: Vec<Question>, quiz_file: String) -> Self {
        Self {
            game: QuizGame::new(questions),
            timer: Instant::now(),
            time_per_question: Duration::from_secs(TIME_PER_QUESTION_SECS),
            quiz_file,
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
                    let questions = load_questions(&self.quiz_file).unwrap_or_default();
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
        });
        ctx.request_repaint();
    }
}

fn main() -> eframe::Result<()> {
    assert_invariant!(QUIZ_CONTRACT_HOLDS, check_quiz_invariants().is_ok());
    let args: Vec<String> = env::args().collect();
    let quiz_file = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| DEFAULT_QUIZ.to_string());
    let questions = load_questions(&quiz_file).unwrap_or_default();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Quiz",
        native_options,
        Box::new(move |_cc| Box::new(QuizApp::new(questions, quiz_file))),
    )
}
