use std::cmp::{max, min};
use std::fs::read_dir;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use log::{debug, info, Level, log};
use pleco::{BitMove, Board, Piece, PieceType};
use pleco::board::movegen::{Legal, MoveGen};
use pleco::core::mono_traits::AllGenType;
use rand::random;
use vampirc_uci::Rule::{bestmove, info, stop};
use crate::core::eval::evaluation::{ClassicEvaluator};
use crate::core::eval::{Evaluator};
use crate::core::heuristics::history_heuristics::HistoryHeuristic;
use crate::core::heuristics::killer_moves::KillerMoves;
use crate::core::heuristics::move_ordering::{order_moves};
use crate::core::Limiter;
use crate::core::openings::OpeningBook;
use crate::core::search::extensions::calculate_extensions;
use crate::core::search::transposition_table::{EvalType, TranspositionTable};

#[derive(Default)]
pub struct Statistics {
    pub(crate) evaluated_positions: u32,
    pub(crate) transposition_entries: u32,
    pub(crate) transposition_deletes: u32,
    pub(crate) transposition_uses: u32,
    pub(crate) beta_cutoffs: u32,
    pub(crate) nodes_searched: u32,

}
pub static mut STATISTICS: Statistics = Statistics {
    evaluated_positions: 0,
    transposition_entries: 0,
    transposition_deletes: 0,
    transposition_uses: 0,
    beta_cutoffs: 0,
    nodes_searched: 0,
};

const INFINITY: i32 = 2147483600;
const NEGATIVE_INFINITY: i32 = -INFINITY;
const IMMEDIATE_MATE_SCORE: i32 = 100000;
pub struct Searcher {
    pub opening_book: OpeningBook,
    pub transposition_table: TranspositionTable,
    pub history_heuristics: HistoryHeuristic,
    pub killer_moves: KillerMoves,
    pub best_move_this_iter: BitMove,
    pub has_searched_one_move: bool,
    pub time_control: Instant,
    pub target_depth: u8,
    pub target_time: Duration,
    pub evaluator: Box<dyn Evaluator>,
}
pub const MAX_DEPTH: u8 = 64;
impl Searcher {
    pub fn new() -> Self {
        Self {
            opening_book: OpeningBook::load("/home/victor/RustroverProjects/chessig2/openings/game_database.pgn"),
            transposition_table: TranspositionTable::new(),
            history_heuristics: HistoryHeuristic::new(),
            killer_moves: KillerMoves::new(MAX_DEPTH as usize),
            best_move_this_iter: BitMove::null(),
            has_searched_one_move: false,
            time_control: Instant::now(),
            target_depth: MAX_DEPTH,
            target_time: Duration::MAX,
            evaluator: Box::new(ClassicEvaluator::new()),
        }
    }
    pub fn search(&mut self, board: &mut Board, limiter: Limiter) -> BitMove {
        
        if limiter.is_time() {
            self.target_time = limiter.get_time().unwrap();
        } else {
            self.target_depth = limiter.get_depth().unwrap()
        }
        unsafe {
            STATISTICS = Statistics::default();
        }

        if let Some(m) = self.opening_book.get_next_move() {
            if board.apply_uci_move(&m) {
                let m = board.last_move().unwrap();
                board.undo_move();
                return m;
            }
        }

        self.opening_book.set_disabled();
        self.time_control = Instant::now();
        self.best_move_this_iter = BitMove::null();
        let mut best_eval = NEGATIVE_INFINITY;
        let mut best_move = BitMove::null();
        let mut d: i32 = -1;
        for depth in 0..=self.target_depth {

            d+=1;

            if self.time_control.elapsed() > self.target_time {
                break;
            }

            unsafe {
                STATISTICS=Statistics::default();
            }
            // Call alpha-beta with reduced depth and negated values for minimax
            let eval = self.alpha_beta(board, NEGATIVE_INFINITY, INFINITY, depth, 0, 0, false);


            if eval == -1 && self.time_control.elapsed() > self.target_time && !self.best_move_this_iter.is_null() {
                // best_move = self.best_move_this_iter;
                break;
            }

            if self.has_searched_one_move {
                best_move = self.best_move_this_iter;
                best_eval = eval;
                self.has_searched_one_move = false;
                self.best_move_this_iter = BitMove::null();
                if is_mate_eval(eval) {
                    // println!("Found mate at depth: {}", depth);
                    break;
                }
            }

        }

        unsafe {
            let usage: f32 = self.transposition_table.usage();

            if is_mate_eval(best_eval) {
                println!("info depth {} nodes {} score cp {}", d, STATISTICS.nodes_searched, (IMMEDIATE_MATE_SCORE - best_eval).abs())
            } else {
                println!("info depth {} nodes {} score cp {}", d, STATISTICS.nodes_searched, best_eval)
            }
            println!("info nps {}", (STATISTICS.nodes_searched as f64 / self.time_control.elapsed().as_secs_f64()) as u64);
            println!("info hashfull {}",  (usage * 1000.0) as u64);


            println!("Move: {:?}, Score: {}, Depth: {}", self.best_move_this_iter.to_string(), best_eval, d);
            println!("Evaluated: {} positions", STATISTICS.evaluated_positions);
            println!("Beta cut offs: {}", STATISTICS.beta_cutoffs);

            println!("Transposition table usage at {:.2}%", usage * 100.0);
            println!("Transpositions entries: {}", STATISTICS.transposition_entries);
            println!("Transposition uses: {}", STATISTICS.transposition_uses);
            println!("Nodes searched: {}", STATISTICS.nodes_searched);
            println!("Speed at: {:.3}Mn/s", (STATISTICS.nodes_searched as f64 / self.time_control.elapsed().as_secs_f64()) / 1000000.0);
        }

        println!("Took about {:?} to find solution", self.time_control.elapsed());
        best_move
    }

    pub fn alpha_beta(&mut self, board: &mut Board, mut alpha: i32, beta: i32, mut depth: u8, ply_from_root: u8, num_extensions: u8, can_do_null_move: bool) -> i32 {
        unsafe { STATISTICS.nodes_searched += 1; }

        /// THIS TWO CHECKS BEFORE DEPTH CHECK!
        if board.stalemate() || board.fifty_move_rule() || board.threefold_repetition() {
            return 0;
        }
        if board.checkmate() {
            return -(IMMEDIATE_MATE_SCORE - ply_from_root as i32);
        }


        if depth == 0 {
            let quiescence_classic_eval = self.quiescence_search(board, alpha, beta, ply_from_root + 1);
            return quiescence_classic_eval;
            // return self.evaluator.evaluate_board(board);
        }

        if let Some(tt_eval) = self.transposition_table.lookup_eval(board, depth, ply_from_root, alpha, beta) {
            unsafe {
                STATISTICS.transposition_uses += 1;
            }
        
            if ply_from_root == 0 {
                if let Some(stored_move) = self.transposition_table.get_stored_move(board) {
                    board.apply_move(stored_move);
                    let legal = MoveGen::generate::<Legal, AllGenType>(board);
        
                    let mut found_draw = false;
                    for m2 in legal {
                        board.apply_move(m2);
                        if board.fifty_move_rule() || board.threefold_repetition() {
                            found_draw = true;
                            board.undo_move();
                            break;
                        }
                        board.undo_move();
                    }
                    board.undo_move();
        
                    if !found_draw {
                        self.best_move_this_iter = stored_move;
                        self.has_searched_one_move = true;
                        return tt_eval;
                    }
                }
            } else {
                let mut found_draw = false;
                for m in board.generate_moves() {
                    board.apply_move(m);
                    if board.threefold_repetition() || board.fifty_move_rule() {
                        found_draw = true;
                        board.undo_move();
                        break;
                    }
                    board.undo_move();
                }
                if !found_draw {
                    return tt_eval;
                }
            }
        }
        if ply_from_root > 3 {
            let eval = self.evaluator.evaluate_board(board);
            let margin = 50;
            if eval - margin >= beta {
                return eval - margin;
            }
        }
        
        let mut best_move = BitMove::null();

        let sorted_moves = order_moves(board, self, depth, false);
        let mut eval_bound = EvalType::UpperBound;
        for (i, &mve) in sorted_moves.iter().enumerate() {
            if self.time_control.elapsed() > self.target_time {
                return -1;
            }
            
            let mut eval = -INFINITY;
            let mut full_search = true;

            board.apply_move(mve);
            
            let extension = calculate_extensions(board, mve, num_extensions);
            const REDUCED_DEPTH: u8 = 2;
            if i > 4 && extension == 0 && !board.in_check() && depth >= 3 {
                eval = -self.alpha_beta(board, -beta, -alpha, depth - 1 - REDUCED_DEPTH, ply_from_root + 1, num_extensions, true);

                full_search = eval > alpha;
            }
            if full_search {
                if i == 0 {
                    // Full window search for the first move
                    eval = -self.alpha_beta(board, -beta, -alpha, depth - 1 + extension, ply_from_root + 1, extension + num_extensions, true);
                } else {
                    // Null window search for subsequent moves
                    eval = -self.alpha_beta(board, -alpha - 1, -alpha, depth - 1, ply_from_root + 1, num_extensions + extension, true);

                    // If the null window search fails high, perform a full search
                    if eval > alpha && eval < beta {
                        eval = -self.alpha_beta(board, -beta, -alpha, depth - 1, ply_from_root + 1, num_extensions + extension, true);
                    }
                }
            }
            board.undo_move();
            if eval == -1 && self.time_control.elapsed() > self.target_time {
                return -1;
            }

            // if is_mate_eval(eval) {
            //     depth = 1;
            // }
            
            if eval >= beta {
                self.transposition_table.store(board, depth, ply_from_root, eval, EvalType::LowerBound, mve);
                unsafe {
                    STATISTICS.beta_cutoffs+=1;
                }
                // if !is_capture(mve, board) {
                //     self.killer_moves.add_killer(depth, mve);
                // }

                return beta;  // Beta cut-off
            }
            if eval > alpha {
                eval_bound = EvalType::Exact;
                best_move = mve;

                alpha = eval;
                self.history_heuristics.update(mve, depth);

                if ply_from_root == 0 {
                    self.best_move_this_iter = mve;
                    self.has_searched_one_move = true;
                }
            }
        }
        self.transposition_table.store(board, depth, ply_from_root, alpha, eval_bound, best_move);
        alpha
    }
    pub fn quiescence_search(&mut self, board: &mut Board, mut alpha: i32, beta: i32, ply_from_root: u8) -> i32 {
        let eval = self.evaluator.evaluate_board(board);

        // Check for terminal conditions (checkmate, stalemate)
        if board.checkmate() {
            return -(IMMEDIATE_MATE_SCORE - ply_from_root as i32);
        } else if board.threefold_repetition() || board.fifty_move_rule() || board.stalemate() {
            return 0; // Draw
        }
        unsafe { STATISTICS.evaluated_positions += 1 }
        // If the static evaluation is already better than beta, prune the search
        if eval >= beta {
            unsafe { STATISTICS.beta_cutoffs += 1 }
            return beta;
        }

        // Update alpha with the static evaluation if it is better
        alpha = max(alpha, eval);


        // Sort captures (you might want to use a more sophisticated move ordering, like MVV-LVA)
        let moves = order_moves(board, self, 0,true);

        // Evaluate the sorted captures
        for mve in moves {
            board.apply_move(mve);
            let score = -self.quiescence_search(board, - beta, - alpha, ply_from_root + 1);
            board.undo_move();
                
            if score >= beta {
                unsafe { STATISTICS.beta_cutoffs += 1; }
                return beta; // Beta cutoff
            }

            alpha = max(alpha, score);
        }

        alpha
    }
}

pub fn is_capture(m: BitMove, context: &Board) -> bool {
    let destinaton = m.get_dest();
    context.piece_at_sq(destinaton) != Piece::None
}
pub fn is_mate_eval(eval: i32) -> bool {
    (eval - IMMEDIATE_MATE_SCORE).abs() < 100
}