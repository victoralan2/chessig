use std::fs;
use std::io::read_to_string;
use std::time::Instant;
use pgn_reader::{BufferedReader, SanPlus, Visitor};
use rand::distr::Distribution;
use pleco::BitMove;
use rand::prelude::ThreadRng;
use rand::thread_rng;
use shakmaty::{Board, CastlingMode, Chess, Position};

struct MoveGetter {
    pub moves: Vec<String>,
    pub current_position: shakmaty::Chess,
}
impl MoveGetter {
    pub fn new() -> Self {
        Self{
            moves: vec![],
            current_position: Chess::new(),
        }
    }
}
impl Visitor for MoveGetter {
    type Result = Vec<String>;
    fn san(&mut self, _san_plus: SanPlus) {
        if self.moves.len() > 10 {
            return;
        }
        let m = _san_plus.san.to_move(&self.current_position).unwrap();
        let uci_move = m.to_uci(CastlingMode::Standard);
        self.current_position.play_unchecked(&m);
        self.moves.push(uci_move.to_string());
    }

    fn end_game(&mut self) -> Self::Result {
        self.moves.clone()
    }
}
#[derive(Clone)]
struct CurrentPosition {
    pub last_move: String,
    pub child_positions: Vec<CurrentPosition>,
    pub count: usize,
    pub depth: usize,
}
impl CurrentPosition {
    pub fn new(games: Vec<Vec<String>>, depth: usize) -> Self {
        let mut root = CurrentPosition {
            last_move: "startpos".to_string(),
            child_positions: Vec::new(),
            count: games.len(),
            depth,
        };

        for game in games {
            root.add_game(&game, 0);
        }

        root
    }
    fn add_game(&mut self, game: &[String], move_index: usize) {
        if move_index >= game.len() {
            return;
        }

        let current_move = &game[move_index];

        if let Some(child) = self.child_positions.iter_mut().find(|c| c.last_move == *current_move) {
            child.count += 1;
            child.add_game(game, move_index + 1);
        } else {
            let mut new_child = CurrentPosition {
                last_move: current_move.clone(),
                child_positions: Vec::new(),
                count: 1,
                depth: self.depth + 1,
            };
            new_child.add_game(game, move_index + 1);
            self.child_positions.push(new_child);
        }
    }
}
pub struct OpeningBook {
    enabled: bool,
    start_position: CurrentPosition,
    current_position: CurrentPosition,
    rng: ThreadRng,
}

impl OpeningBook {
    pub fn load(path: &str) -> Self {
        let games = Self::get_all_games(path.to_string());
        let current_position = CurrentPosition::new(games, 0);
        Self { start_position: current_position.clone(), current_position, rng: thread_rng(), enabled: true }
    }
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    fn get_all_games(path: String) -> Vec<Vec<String>> {
        let pgn_string = fs::read_to_string(path).unwrap();
        let mut reader = BufferedReader::new_cursor(&pgn_string[..]);

        let mut move_getter = MoveGetter::new();
        let mut games = vec![];
        while let Some(moves) = reader.read_game(&mut move_getter).ok().flatten() {
            games.push(moves);
            move_getter = MoveGetter::new();
        }
        games
    }
    pub fn get_next_move(&mut self) -> Option<String> {
        if !self.enabled { 
            return None
        }
        let possible_moves = &self.current_position.child_positions;
        if possible_moves.is_empty() {
            return None;
        }
        let weighted_index = rand::distr::WeightedIndex::new(possible_moves.iter().map(|p| p.count * p.count)).unwrap();
        let mve = possible_moves[weighted_index.sample(&mut self.rng)].clone();
        let r = Some(mve.last_move.clone());
        r
    }
    pub fn set_disabled(&mut self) {
        self.enabled = false;
    }
    pub fn set_enabled(&mut self) {
        self.enabled = true;
    }
    pub fn reset(&mut self) {
        self.current_position = self.start_position.clone();
    }
    pub fn apply_move(&mut self, m: String) -> bool {
        let m: Vec<&CurrentPosition> = self.current_position.child_positions.iter().filter(|p| p.last_move == m).collect();
        if let Some(&pos) = m.first() { 
            self.current_position = pos.clone();
            true
        } else { false }    
    }
}

fn print_tree(node: &CurrentPosition, indent: usize) {
    println!("{}{} (Depth: {}, Games: {})",
             "  ".repeat(indent),
             node.last_move,
             node.depth,
             node.count);
    for child in &node.child_positions {
        print_tree(child, indent + 1);
    }
}