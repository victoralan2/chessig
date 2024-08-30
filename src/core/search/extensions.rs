use pleco::{BitMove, Board, PieceType, Rank};

pub fn calculate_extensions(board: &Board, move_played: BitMove, num_extensions: u8) -> u8 {
    const MAX_EXTENSIONS: u8 = 16;
    let mut extensions = 0u8;
    // let target_rank = move_played.get_dest().rank();
    // if board.piece_at_sq(move_played.get_src()).type_of() == PieceType::P 
    //     && (target_rank == Rank::R2 || target_rank == Rank::R7) {
    //     extensions += 3;
    // }
    extensions += board.in_check() as u8;

    if extensions + num_extensions > MAX_EXTENSIONS {
        extensions = 0;
    }
    extensions
}