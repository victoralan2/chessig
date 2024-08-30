use std::time::Duration;
use pleco::{BitMove, Board};
use pleco::board::movegen::{Legal, MoveGen};
use pleco::core::mono_traits::AllGenType;
use pleco::Player::{Black, White};
use crate::core::{Limiter, move_to_san};
use crate::core::search::search::Searcher;


pub static mut USE_RFP: bool = false;

pub fn self_play_test(white_rsp: bool) -> i32 {
    
    let mut board = Board::start_pos();

    let mut searcher_rsp = Searcher::new();
    let mut searcher_non_rsp = Searcher::new();


    let mut i = 1;

    let mut pgn_str: String = "".into();
    while !board.stalemate() && !board.checkmate() && !board.threefold_repetition() && !board.fifty_move_rule() {
        let mut m = BitMove::null();
        if (i - 1) % 2 == 1 - (white_rsp as usize)  {
            m = searcher_rsp.search(&mut board, Limiter::time(Duration::from_secs_f32(2.0)));
            
            unsafe { USE_RFP = true }
        } else {
            m = searcher_non_rsp.search(&mut board, Limiter::time(Duration::from_secs_f32(2.0)));
            
            unsafe { USE_RFP = false }
        }
        
        let legal = MoveGen::generate::<Legal, AllGenType>(&board);
        if legal.iter().copied().collect::<Vec<BitMove>>().contains(&m) {
            let mv = move_to_san(&board, &m);
            if i % 2 == 1 {
                pgn_str.push_str(format!("{}. ", (i+1)/2).as_str());
            }
            pgn_str.push_str(format!("{} ", mv).as_str());
            board.apply_move(m);
            searcher_rsp.opening_book.apply_move(m.to_string());
            searcher_non_rsp.opening_book.apply_move(m.to_string());
            
            i += 1;
        } else {
            println!("{}", pgn_str);
            println!("A bad move has been played: {}", m);
            println!("Position {}", board.fen());
            break
        }
    }
    println!("{}", pgn_str);
    
    if board.checkmate() {
        if white_rsp {
            return if board.turn() == White {
                println!("Won NON RSP");
                -1
            } else {
                println!("Won RSP");
                1
            }
        } else if board.turn() == Black {
            println!("Won RSP");
            return 1;
        } else {
            println!("Won NON RSP");
            return -1;
        }
    }
    return 0;
        
    // let mut f= fs::File::create("./game.pgn").unwrap();
    // f.write_all(pgn_str.as_utf8_bytes()).unwrap();
}