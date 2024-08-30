use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::{Display};
use std::hash::{Hash, RandomState};
use std::mem::{size_of, size_of_val};
use pleco::{BitMove, Board, PieceType};
use rand::prelude::IteratorRandom;
use rand::rngs::{SmallRng, ThreadRng};
use crate::core::search::search::{is_mate_eval, STATISTICS};

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum EvalType {
    UpperBound,
    Exact,
    LowerBound
}
#[derive(Clone)]
pub struct TranspositionTableEntry {
    pub hash_key: u64,
    pub best_move: BitMove,
    pub depth: u8,
    pub eval: i32,
    pub eval_type: EvalType,
}
impl Default for TranspositionTableEntry {
    fn default() -> Self {
        Self {
            hash_key: 0,
            best_move: BitMove::null(),
            depth: 0,
            eval: 0,
            eval_type: EvalType::Exact,
        }
    }
}
const TRANSPOSITION_TABLE_SIZE_MB: usize = 64;
const TRANSPOSITION_TABLE_ENTRIES: usize = TRANSPOSITION_TABLE_SIZE_MB * 1024 * 1024 / size_of::<TranspositionTableEntry>();


pub struct FixedSizeHashMap<K: Eq + Hash + Clone, V, S = RandomState> {
    map: HashMap<K, V, S>,
    rand: SmallRng,
}

impl<K: Eq + Hash + Clone, V> FixedSizeHashMap<K, V> {
    pub fn new(capacity: usize) -> Self {

        Self {
            map: HashMap::with_capacity(capacity),
            rand: SmallRng::from_thread_rng()
        }
    }
    pub fn insert(&mut self, key: K, value: V) {
        if self.map.len() >= self.map.capacity() {
            self.evict_random_entry()
        }
        unsafe { STATISTICS.transposition_entries+=1 }

        self.map.insert(key, value);
    }
    pub fn get(&self, key: &K) -> Option<&V> {
        self.map.get(key)
    }
    fn evict_random_entry(&mut self) {
        unsafe { STATISTICS.transposition_deletes+=1 }
        unsafe { STATISTICS.transposition_entries-=1 }

        let keys: Vec<K> = self.map.keys().cloned().collect();

        // Check if there are any keys to evict
        if let Some(key_to_evict) = keys.iter().choose(&mut self.rand) {
            self.map.remove(key_to_evict);
        }
    }
    pub fn get_size(&self) -> usize {
        self.map.len() * ( size_of::<V>() + size_of::<K>() )
    }
    pub fn entry(&mut self, k: K) -> Entry<'_, K, V> {
        self.map.entry(k)
    }
}
pub struct TranspositionTable {
    entries: FixedSizeHashMap<u64, TranspositionTableEntry>,
    // entries: [TranspositionTableEntry; TRANSPOSITION_TABLE_ENTRIES],
}

impl TranspositionTable {
    pub(crate) fn new() -> TranspositionTable {
        TranspositionTable {
            entries: FixedSizeHashMap::new(TRANSPOSITION_TABLE_ENTRIES),
        }
    }
    pub fn get_size(&self) -> usize {
        self.entries.get_size()
    }
    pub fn store(&mut self, board: &Board, depth: u8, ply_searched: u8, eval: i32, eval_type: EvalType, best_move: BitMove) {
        let hash_key = board.zobrist();
        let entry = TranspositionTableEntry {
            hash_key,
            best_move,
            depth,
            eval: Self::correct_mate_eval_store(eval, ply_searched),
            eval_type,
        };
        if let Some(t) = self.entries.get(&entry.hash_key) {
            if t.depth > depth {
                return;
            }
        }

        self.entries.insert(entry.hash_key, entry);
    }
    pub fn lookup_eval(&self, board: &Board, depth: u8, ply_from_root: u8, alpha: i32, beta: i32) -> Option<i32> {
        if let Some(entry) = self.entries.get(&board.zobrist()) {
            if entry.depth >= depth {
                let eval = Self::correct_mate_eval_retrive(entry.eval, ply_from_root);
                if entry.eval_type == EvalType::Exact {
                    return Some(eval);
                }
                if entry.eval_type == EvalType::UpperBound && eval <= alpha {
                    return Some(eval);
                }
                if entry.eval_type == EvalType::LowerBound && eval >= beta {
                    return Some(eval);
                }
            }
        }
        None
    }
    pub fn get_stored_move(&self, board: &Board) -> Option<BitMove> {
        self.entries.get(&board.zobrist()).map(|x| x.best_move)
    }
    fn correct_mate_eval_store(eval: i32, ply_searched: u8) -> i32 {
        if is_mate_eval(eval) {
            let sign = eval.signum();
            return (eval * sign + ply_searched as i32) * sign;
        }
        eval
    }
    fn correct_mate_eval_retrive(eval: i32, ply_searched: u8) -> i32 {
        if is_mate_eval(eval) {
            let sign = eval.signum();
            return (eval * sign - ply_searched as i32) * sign;
        }
        eval
    }
    pub fn usage(&self) -> f32 {
        let mut used = 0;
        let len = self.entries.map.capacity();
        for (_, e) in &self.entries.map {
            if e.hash_key != 0 {
                used += 1;
            }
        }
        used as f32 / len as f32
    }
}