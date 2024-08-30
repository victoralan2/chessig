use pleco::BitMove;

pub struct KillerMoves {
    killers: Vec<[Option<BitMove>; 2]>,  // Store two best "killer" moves for each depth
}

impl KillerMoves {
    pub fn new(max_depth: usize) -> Self {
        Self {
            killers: vec![[None, None]; max_depth],
        }
    }

    pub fn add_killer(&mut self, depth: u8, mv: BitMove) {
        let depth = depth as usize;
        if self.killers[depth][0] != Some(mv) {
            self.killers[depth][1] = self.killers[depth][0];
            self.killers[depth][0] = Some(mv);
        }
    }

    pub fn is_killer(&self, depth: u8, mv: BitMove) -> bool {
        self.killers[depth as usize].contains(&Some(mv))
    }
}