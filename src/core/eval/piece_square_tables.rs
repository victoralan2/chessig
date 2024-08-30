use pleco::{BitBoard, Board, PieceType, Player, SQ};
use crate::mathutils::lerp;

pub const PAWN_TABLE: [i32; 64] =
	[0,  0,  0,  0,  0,  0,  0,  0,
	50, 50, 50, 50, 50, 50, 50, 50,
	10, 10, 20, 30, 30, 20, 10, 10,
	5,  5, 10, 25, 25, 10,  5,  5,
	0,  0,  0, 20, 20,  0,  0,  0,
	5, -5,-10,  0,  0,-10, -5,  5,
	5, 10, 10,-20,-20, 10, 10,  5,
	0,  0,  0,  0,  0,  0,  0,  0];

pub const KNIGHT_TABLE: [i32; 64] =
	[-50,-40,-30,-30,-30,-30,-40,-50,
	-40,-20,  0,  0,  0,  0,-20,-40,
	-30,  0, 10, 15, 15, 10,  0,-30,
	-30,  5, 15, 20, 20, 15,  5,-30,
	-30,  0, 15, 20, 20, 15,  0,-30,
	-30,  5, 10, 15, 15, 10,  5,-30,
	-40,-20,  0,  5,  5,  0,-20,-40,
	-50,-40,-30,-30,-30,-30,-40,-50,];

pub const BISHOP_TABLE: [i32; 64] =
	[-20,-10,-10,-10,-10,-10,-10,-20,
	-10,  0,  0,  0,  0,  0,  0,-10,
	-10,  0,  5, 10, 10,  5,  0,-10,
	-10,  5,  5, 10, 10,  5,  5,-10,
	-10,  0, 10, 10, 10, 10,  0,-10,
	-10, 10, 10, 10, 10, 10, 10,-10,
	-10,  5,  0,  0,  0,  0,  5,-10,
	-20,-10,-10,-10,-10,-10,-10,-20,];

pub const ROOK_TABLE: [i32; 64] =
	[  0,  0,  0,  0,  0,  0,  0,  0,
	5, 10, 10, 10, 10, 10, 10,  5,
	-5,  0,  0,  0,  0,  0,  0, -5,
	-5,  0,  0,  0,  0,  0,  0, -5,
	-5,  0,  0,  0,  0,  0,  0, -5,
	-5,  0,  0,  0,  0,  0,  0, -5,
	-5,  0,  0,  0,  0,  0,  0, -5,
	0,  0,  0,  5,  5,  0,  0,  0];

pub const QUEEN_TABLE: [i32; 64] =
	[-20,-10,-10, -5, -5,-10,-10,-20,
	-10,  0,  0,  0,  0,  0,  0,-10,
	-10,  0,  5,  5,  5,  5,  0,-10,
	-5,  0,  5,  5,  5,  5,  0, -5,
	0,  0,  5,  5,  5,  5,  0, -5,
	-10,  5,  5,  5,  5,  5,  0,-10,
	-10,  0,  5,  0,  0,  0,  0,-10,
	-20,-10,-10, -5, -5,-10,-10,-20];

pub const KING_MID_TABLE: [i32; 64] =
	[-30,-40,-40,-50,-50,-40,-40,-30,
	-30,-40,-40,-50,-50,-40,-40,-30,
	-30,-40,-40,-50,-50,-40,-40,-30,
	-30,-40,-40,-50,-50,-40,-40,-30,
	-20,-30,-30,-40,-40,-30,-30,-20,
	-10,-20,-20,-20,-20,-20,-20,-10,
	20, 20,  0,  0,  0,  0, 20, 20,
	20, 30, 10,  0,  0, 10, 30, 20];

pub const KING_END_TABLE: [i32; 64] =
	[-50,-40,-30,-20,-20,-30,-40,-50,
	-30,-20,-10,  0,  0,-10,-20,-30,
	-30,-10, 20, 30, 30, 20,-10,-30,
	-30,-10, 30, 40, 40, 30,-10,-30,
	-30,-10, 30, 40, 40, 30,-10,-30,
	-30,-10, 20, 30, 30, 20,-10,-30,
	-30,-30,  0,  0,  0,  0,-30,-30,
	-50,-30,-30,-30,-30,-30,-30,-50];

pub struct PieceSquareTables();

impl PieceSquareTables {
	///
	/// bitboard: The bitboard of the given piece
	/// piece: The piece to check
	/// color: The color of the piece
	/// endgame_weight: How close is the board to endgame (0, 1)
	pub fn eval_piece(bitboard: &BitBoard, piece: PieceType, color: Player, endgame_weight: f32) -> f32 {
		let mut eval = 0.0;
		let mut endgame_importance = 0.;
		bitboard.for_each(|square| {
			match piece {
				PieceType::P => endgame_importance = 0.5,
				PieceType::N => endgame_importance = 0.2,
				PieceType::B => endgame_importance = 0.5,
				PieceType::R => endgame_importance = 0.3,
				PieceType::Q => endgame_importance = 0.8,
				PieceType::K =>  endgame_importance = 1.0,
				_ => {}
			}
			eval += PieceSquareTables::get_value_square(square, piece, color, endgame_weight);
		});
		eval
	}
	pub fn eval_king(square: SQ, color: Player, endgame_weight: f32) -> f32 {
		
		PieceSquareTables::get_value_square(square, PieceType::K, color, endgame_weight)
	}
	fn get_value_square(square: SQ, piece: PieceType, color: Player, endgame_weight: f32) -> f32{
		let mut square_index = square.0 as usize;
		if color == Player::White {
			square_index = 63 - square_index;
		}
		match piece {
			PieceType::P => PAWN_TABLE[square_index] as f32,
			PieceType::N => KNIGHT_TABLE[square_index] as f32,
			PieceType::B => BISHOP_TABLE[square_index] as f32,
			PieceType::R => ROOK_TABLE[square_index] as f32,
			PieceType::Q => QUEEN_TABLE[square_index] as f32,
			PieceType::K => {
				let king_mid = KING_MID_TABLE[square_index];
				let king_end = KING_END_TABLE[square_index];
				lerp(king_mid as f32, king_end as f32, endgame_weight)
			}
			_ => {
				0.0
			}
		}
	}
}