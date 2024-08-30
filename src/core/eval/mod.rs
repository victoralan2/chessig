use pleco::{Board, Player};
use crate::core::eval::evaluation::ClassicEvaluator;

pub mod evaluation;
mod piece_square_tables;

pub trait Evaluator {
    fn evaluate_board(&mut self, board: &Board) -> i32;
}
