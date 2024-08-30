use std::str::FromStr;
use std::time::Duration;
use pleco::{BitMove, Board};
use shakmaty::{CastlingMode, FromSetup, Setup};
use shakmaty::fen::Fen;

pub mod search;
pub mod eval;
pub mod heuristics;
pub mod openings;


/// This is really slow
pub fn move_to_san(board: &Board, m: &BitMove) -> String {
    let c = shakmaty::Chess::from_setup(Setup::from(Fen::from_str(&board.fen()).unwrap()), CastlingMode::Standard).unwrap();
    let san_move = shakmaty::san::San::from_move(&c, &shakmaty::uci::UciMove::from_str(&m.stringify()).unwrap().to_move(&c).unwrap()).to_string();
    san_move
}

pub struct Limiter {
    time: Option<std::time::Duration>,
    depth: Option<u8>,
}
impl Limiter {
    pub fn time(duration: std::time::Duration) -> Self {
        Self {
            time: Some(duration),
            depth: None,
        }
    }
    pub fn depth(depth: u8) -> Self {
        Self {
            time: None,
            depth: Some(depth),
        }
    }
    pub fn both(depth: u8, time: Duration) -> Self {
        Self {
            time: Some(time),
            depth: Some(depth),
        }
    }
    pub fn is_time(&self) -> bool {
        self.time.is_some()
    }
    pub fn is_depth(&self) -> bool {
        self.depth.is_some()
    }
    pub fn get_time(&self) -> Option<Duration> {
        self.time.clone()
    }
    pub fn get_depth(&self) -> Option<u8> {
        self.depth.clone()
    }
}