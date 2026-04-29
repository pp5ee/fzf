use crate::algo::MatchResult;
use crate::item::Item;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Rank {
    pub score: i32,
    pub index: u32,
    pub length: u16,
}

impl Rank {
    pub fn new(score: i32, index: u32, length: u16) -> Self {
        Self {
            score,
            index,
            length,
        }
    }
}

impl PartialEq for Rank {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score && self.index == other.index && self.length == other.length
    }
}

impl Eq for Rank {}

impl PartialOrd for Rank {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Rank {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.score
            .cmp(&other.score)
            .then_with(|| self.length.cmp(&other.length))
            .then_with(|| self.index.cmp(&other.index).reverse())
    }
}

#[derive(Debug, Clone)]
pub struct Result {
    pub item: Arc<Item>,
    pub rank: Rank,
    pub match_result: MatchResult,
    pub offsets: Vec<(u16, u16)>,
}

impl Result {
    pub fn new(item: Arc<Item>, rank: Rank, match_result: MatchResult) -> Self {
        Self {
            item,
            rank,
            match_result,
            offsets: Vec::new(),
        }
    }

    pub fn with_offsets(mut self, offsets: Vec<(u16, u16)>) -> Self {
        self.offsets = offsets;
        self
    }
}

#[derive(Debug, Clone)]
pub struct Selection {
    pub current: usize,
    pub items: Vec<Arc<Item>>,
}

impl Selection {
    pub fn new() -> Self {
        Self {
            current: 0,
            items: Vec::new(),
        }
    }

    pub fn push(&mut self, item: Arc<Item>) {
        self.items.push(item);
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.current = 0;
    }

    pub fn get(&self, index: usize) -> Option<&Arc<Item>> {
        self.items.get(index)
    }

    pub fn current(&self) -> Option<&Arc<Item>> {
        self.items.get(self.current)
    }
}

impl Default for Selection {
    fn default() -> Self {
        Self::new()
    }
}
