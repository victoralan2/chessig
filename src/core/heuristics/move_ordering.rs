use pleco::{Board, MoveList};
use pleco::board::movegen::{Legal, MoveGen};
use pleco::core::GenTypes;
use pleco::core::mono_traits::{AllGenType, CapturesGenType};
use rand::random;
use crate::core::eval::evaluation::ClassicEvaluator;
use crate::core::search::search::{Searcher};
use crate::core::search::transposition_table::TranspositionTable;

// Function to order moves based on captures
pub fn order_moves(board: &Board, searcher: &Searcher, depth: u8, quiesence_search: bool) -> MoveList {
    let mut moves;
    let transposition_table = &searcher.transposition_table;
    if quiesence_search {
        moves = board.generate_moves_of_type(GenTypes::Captures)
    } else {
        moves = board.generate_moves();
    }

    let best_move_tt = transposition_table.get_stored_move(board);
    moves.sort_by_key(|&m|{
        let mut transposition_table_value = 0;
        if Some(m) == best_move_tt {
            transposition_table_value += 9999999;
        }
        let is_killer = searcher.killer_moves.is_killer(depth, m);
        let history = searcher.history_heuristics.get_score(m);
        let killer = if is_killer { 99999 } else { 0 };

        // MVV-LVA
        let victim_value = ClassicEvaluator::capture_value(board, m);
        let moved_piece = board.piece_at_sq(m.get_src());
        let attacker_value = ClassicEvaluator::piece_value(moved_piece.type_of());
        let mut mvv_lva_score = victim_value * 10 - attacker_value;
        if victim_value == 0 {
            mvv_lva_score = 0;
        }

        -(killer + history + mvv_lva_score + transposition_table_value)
    });
    moves
}