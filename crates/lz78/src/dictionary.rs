use std::collections::HashMap;

pub struct Dictionary {
    nodes: Vec<(usize, u8)>,
    index: HashMap<Vec<u8>, usize>,
    strategy: PruningStrategy
}

#[derive(Debug, Clone)]
pub enum PruningStrategy {
    /// never prune, dictionary grows indefinitely
    Never,
    /// when dictionary reaches specified max length, drop whole dictionary
    Drop(u64),
    /// when dictionary reaches specified max length, freeze dictionary
    Freeze(u64)
}

impl From<i64> for PruningStrategy {
    fn from(value: i64) -> Self {
        match value {
            0 => PruningStrategy::Never,
            0.. => PruningStrategy::Drop(value as u64),
            ..0 => PruningStrategy::Freeze(value.abs() as u64),
        }
    }
}

impl Into<i64> for PruningStrategy {
    fn into(self) -> i64 {
        match self {
            PruningStrategy::Never => 0,
            PruningStrategy::Drop(value) => value as i64,
            PruningStrategy::Freeze(value) => -(value as i64),
        }
    }
}

impl Dictionary {
    pub fn new(strategy: PruningStrategy) -> Self {
        Self { nodes: Vec::new(), index: HashMap::new(), strategy }
    }

    pub fn add(&mut self, word: &[u8]) -> Option<(usize, u8)> {
        if let PruningStrategy::Drop(max_len) = self.strategy {
            if self.nodes.len() >= max_len as usize {
                self.nodes.clear();
                self.index.clear();
            }
        } else if let PruningStrategy::Freeze(max_len) = self.strategy {
            if self.nodes.len() >= max_len as usize {
                return Some((0, *word.last().unwrap()))
            }
        }

        if self.index.contains_key(word) {
            return None;
        }

        let entry = (!word.is_empty()).then(|| self.index.get(&word[0..(word.len() - 1)])).flatten().copied();

        let last_part = *word.last().unwrap();

        let v = (entry.unwrap_or(0), last_part);

        self.nodes.push(v);
        self.index.insert(word.to_vec(), self.nodes.len());

        Some(v)
    }

    pub fn get(&self, mut index: usize) -> Option<Vec<u8>> {
        let mut word_buf = vec![];
        while let Some(&(next_index, word)) = self.nodes.get(index - 1) {
            word_buf.push(word);
            index = next_index;

            if next_index == 0 {
                break;
            }
        }

        if index != 0 {
            None
        } else {
            word_buf.reverse();
            Some(word_buf)
        }
    }

    pub fn find(&self, word: &[u8]) -> Option<usize> {
        self.index.get(word).copied()
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }
}
