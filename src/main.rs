use std::thread::sleep;
use std::time::Duration;
use pleco::{Board, Piece};
use testing::perf_test::perf_test;
use crate::core::eval::Evaluator;
use crate::core::search::search::Searcher;
use crate::logs::init_log;
use crate::uci::uci_loop;

mod uci;
mod logs;
mod mathutils;
mod core;
mod testing;

fn main() {
    // perf_test();
    init_log();
    uci_loop().ok();
}
