use pleco::{BitMove, Board, Piece, PieceType, Player};
use crate::core::eval::Evaluator;
use crate::core::eval::piece_square_tables::PieceSquareTables;

pub const PAWN_VALUE: i32 = 100;
pub const KNIGHT_VALUE: i32 = 300;
pub const BISHOP_VALUE: i32 = 320;
pub const ROOK_VALUE: i32 = 500;
pub const QUEEN_VALUE: i32 = 900;
pub const ENDGAME_MATERIAL_START: i32 = ROOK_VALUE * 2 + BISHOP_VALUE + KNIGHT_VALUE;


pub struct ClassicEvaluator;

impl Evaluator for ClassicEvaluator {
    fn evaluate_board(&mut self, board: &Board) -> i32 {
        let perspective = if board.turn() == Player::White { 1 } else { -1 };


        let white_material = ClassicEvaluator::count_material(&board, Player::White);
        let black_material = ClassicEvaluator::count_material(&board, Player::Black);

        let white_material_without_pawns = white_material - board.count_piece(Player::White, PieceType::P) as i32;
        let black_material_without_pawns = black_material - board.count_piece(Player::Black, PieceType::P) as i32;

        let white_endgame_weight = Self::endgame_phase_weight(white_material_without_pawns);
        let black_endgame_weight = Self::endgame_phase_weight(black_material_without_pawns);

        let white_piece_square = (Self::evaluate_piece_square(board, white_endgame_weight, Player::White) * 2f32).round() as i32;
        let black_piece_square = (Self::evaluate_piece_square(board, black_endgame_weight, Player::Black) * 2f32).round() as i32;
        let mut white_eval = 0;
        let mut black_eval = 0;

        white_eval += white_material;
        white_eval += white_piece_square;

        black_eval += black_material;
        black_eval += black_piece_square;
        let eval = (white_eval - black_eval) * perspective;
        eval
    }
}
impl ClassicEvaluator {
    pub fn new() -> Self {
        Self{}
    }
    fn count_material(board: &Board, color: Player) -> i32 {
        let mut material = 0;

        material += board.count_piece(color, PieceType::P) as i32 * PAWN_VALUE;
        material += board.count_piece(color, PieceType::N) as i32 * KNIGHT_VALUE;
        material += board.count_piece(color, PieceType::B) as i32 * BISHOP_VALUE;
        material += board.count_piece(color, PieceType::R) as i32 * ROOK_VALUE;
        material += board.count_piece(color, PieceType::Q) as i32 * QUEEN_VALUE;

        material
    }
    pub fn capture_value(board: &Board, m: BitMove) -> i32 {
        ClassicEvaluator::piece_value(board.piece_at_sq(m.get_dest()).type_of())
    }
    // pub fn capture_value(game: &Game) -> i32 {
    //     let mut capture_score = 0;
    // 
    //     for mve in MoveGen::new_legal(&game.current_position()) {
    //         if is_capture(&mve, game) {
    //             let target_piece = game.current_position().piece_on(mve.get_dest());
    //             if target_piece.is_some() {
    //                 capture_score += Self::piece_value(target_piece.unwrap()); // Higher value for capturing pieces
    //             }
    //         }
    //     }
    // 
    //     capture_score
    // }
    pub fn piece_value(piece: PieceType) -> i32 {
        match piece {
            PieceType::P => PAWN_VALUE,
            PieceType::N => KNIGHT_VALUE,
            PieceType::B => BISHOP_VALUE,
            PieceType::R => ROOK_VALUE,
            PieceType::Q => QUEEN_VALUE,
            _ => 0
        }
    }
    pub fn evaluate_piece_square(board: &Board, endgame_weight: f32, color: Player) -> f32 {
        let pawns = board.piece_bb(color, PieceType::P);

        let knights = board.piece_bb(color, PieceType::N);

        let bishops = board.piece_bb(color, PieceType::B);

        let rooks = board.piece_bb(color, PieceType::R);

        let queens = board.piece_bb(color, PieceType::Q);

        let king = board.king_sq(color);

        let mut eval: f32 = 0.0;
        eval += PieceSquareTables::eval_piece(&pawns, PieceType::P, color, endgame_weight);
        eval += PieceSquareTables::eval_piece(&knights, PieceType::N, color, endgame_weight);
        eval += PieceSquareTables::eval_piece(&bishops, PieceType::B, color, endgame_weight);
        eval += PieceSquareTables::eval_piece(&rooks, PieceType::R, color, endgame_weight);
        eval += PieceSquareTables::eval_piece(&queens, PieceType::Q, color, endgame_weight);
        eval += PieceSquareTables::eval_king(king, color, endgame_weight);

        eval
    }
    pub fn endgame_phase_weight(material_count_without_pawns: i32) -> f32 {
        const MULTIPLIER: f32 = 1f32 / ENDGAME_MATERIAL_START as f32;
        1f32 - if 1f32 > MULTIPLIER * material_count_without_pawns as f32 { MULTIPLIER * material_count_without_pawns as f32 } else { 1f32 }
    }
}