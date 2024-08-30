use std::collections::HashMap;
use std::mem::size_of;
use pleco::BitMove;
use crate::core::search::transposition_table::FixedSizeHashMap;

pub struct HistoryHeuristic {
    history: FixedSizeHashMap<BitMove, u16>,
}

impl HistoryHeuristic {
    pub fn new() -> Self {
        let size_one_bitmove = size_of::<BitMove>();
        let total_size = 32 * 1024 * 1024;
        Self {
            history: FixedSizeHashMap::new(total_size / size_one_bitmove),
        }
    }

    pub fn update(&mut self, mv: BitMove, depth: u8) {
        let depth = depth as u16;
        let entry = self.history.entry(mv).or_insert(0);
        *entry += depth * depth; // The deeper the move is searched, the more we value it
    }

    pub fn get_score(&self, mv: BitMove) -> i32 {
        *self.history.get(&mv).unwrap_or(&0) as i32
    }
}