use std::time::Instant;
use pleco::Board;
use pleco::board::movegen::{Legal, MoveGen, PseudoLegal};
use pleco::core::mono_traits::AllGenType;


pub fn perft(board: &mut Board, depth: u32) -> u64 {
    let start = Instant::now();
    let move_gen = MoveGen::generate::<Legal, AllGenType>(board);
    let mut nodes = 0;
    for m in move_gen {
        board.apply_move(m);
        let n = _perft(board, depth-1);
        board.undo_move();
        // println!("Found {} nodes with move {}", n, m.stringify());
        nodes+=n;
    }
    let t = start.elapsed();
    println!("Took {:?} to find {} nodes", t, nodes);
    // println!("Speed of about {} knps", (nodes as f64 / t.as_secs_f64()) /  1000f64);
    nodes
}
fn _perft(board: &mut Board, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut nodes = 0;
    let move_gen = MoveGen::generate::<Legal, AllGenType>(board);

    for m in move_gen {
        board.apply_move(m);
        nodes += _perft(board, depth - 1);
        board.undo_move();
    }

    nodes
}