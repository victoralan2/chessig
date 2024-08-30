extern crate vampirc_uci;

use std::cmp::min;
use std::io;
use std::io::{BufRead, stdin, stdout, Write};
use std::ops::Mul;
use std::process::exit;
use std::str::FromStr;
use std::time::{Duration, Instant};
use log::{info};
use pleco::{BitMove, Board, PieceType, Player};
use pleco::core::piece_move::{MoveFlag, PreMoveInfo};
use pleco::File::B;
use vampirc_uci::{parse, parse_one, UciFen, UciMessage, UciTimeControl};
use vampirc_uci::Rule::message;
use crate::core::Limiter;
use crate::core::search::search::{MAX_DEPTH, Searcher};

pub fn uci_loop() -> Result<(), io::Error>{
    let mut board = Board::start_pos();
    const TARGET_DEPTH: u8 = 20;
    const TARGET_TIME: f64 = 4.0;

    println!("Loading engine...");
    let mut searcher = Searcher::new();
    println!("Engine loaded");
    let stdin = stdin();
    let mut stdout = stdout();
    for line in stdin.lines() {
        let msg: UciMessage = parse_one(&line.unwrap());
        match msg {
            UciMessage::Uci => {
                let _ = stdout.write(b"id name CheRs2\nid author Victor Alan\nuciok\n")?;
            }
            UciMessage::Debug(_) => {}
            UciMessage::IsReady => {
                let _ = stdout.write(b"readyok\n")?;
            }
            UciMessage::Register { .. } => {}
            UciMessage::Position { startpos, fen, moves } => {
                println!("{:?}", fen);
                if startpos {
                    searcher.opening_book.set_enabled();
                    board = Board::start_pos();
                } else {
                    println!("Disabled!");
                    searcher.opening_book.set_disabled();
                    board = Board::from_fen(fen.unwrap().0.as_str()).unwrap();
                }
                for mve in moves {
                    if startpos {
                        searcher.opening_book.apply_move(mve.to_string());
                    }
                    board.apply_uci_move(&mve.to_string());
                }
            }
            UciMessage::SetOption { .. } => {}
            UciMessage::UciNewGame => {
                board = Board::start_pos();
                // searcher = Searcher::new(TARGET_DEPTH, TARGET_TIME);
            }
            UciMessage::Stop => {

            }
            UciMessage::PonderHit => {}
            UciMessage::Quit => {
                exit(0);
            }
            UciMessage::Go { time_control, search_control } => {
                let mut target_depth = None;
                if let Some(search_control) = search_control {
                    target_depth = search_control.depth;
                }

                let start = Instant::now();
                let mut time_control = time_control.unwrap_or(UciTimeControl::Infinite);
                let mut selected_move = match time_control {
                    UciTimeControl::Ponder => {
                        continue;
                    }
                    UciTimeControl::Infinite => {
                        if let Some(target_depth) = target_depth {
                            searcher.search(&mut board, Limiter::depth(target_depth))
                        } else {
                            searcher.search(&mut board, Limiter::time(Duration::from_secs(5)))
                        }
                    }
                    UciTimeControl::TimeLeft {
                        white_time,
                        black_time,
                        moves_to_go, ..
                    } => {
                        
                        let mut n_moves= 0;
                        if !searcher.opening_book.is_enabled() {
                            n_moves = min(board.ply() - 10, 10);
                        }

                        let factor = 2 - (n_moves / 10);
                        
                        let target = if board.turn() == Player::White { white_time.unwrap() } else { black_time.unwrap() } / moves_to_go.unwrap() as i32;
                        let time = target.mul(factor as i32);

                        let limiter = if let Some(target_depth) = target_depth {
                            Limiter::both(target_depth, time.to_std().unwrap())
                        } else {
                            Limiter::time(time.to_std().unwrap())
                        };
                        searcher.search(&mut board, limiter)
                    }
                    UciTimeControl::MoveTime(time) => {
                        let limiter = if let Some(target_depth) = target_depth {
                            Limiter::both(target_depth, time.to_std().unwrap())
                        } else {
                            Limiter::time(time.to_std().unwrap())
                        };
                        searcher.search(&mut board, limiter)
                    }
                };

                info!("Took {:?} to select move: {}", start.elapsed(), selected_move.stringify());
                let str_move = selected_move.stringify();
                let response = format!("bestmove {}\n", str_move);

                let _ = stdout.write(response.as_bytes())?;
            }
            _ => {}
        }
        stdout.flush()?;
    }

    Ok(())
}