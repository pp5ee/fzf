use crate::result::{Rank, Result};
use std::sync::Arc;

pub struct Merger {
    lists: Vec<Vec<Result>>,
    sorted: bool,
    final_: bool,
    ascending: bool,
}

impl Merger {
    pub fn new(lists: Vec<Vec<Result>>, sorted: bool, final_: bool) -> Self {
        Self {
            lists,
            sorted,
            final_,
            ascending: true,
        }
    }

    pub fn merge(&mut self) -> Vec<Result> {
        if self.lists.is_empty() {
            return Vec::new();
        }

        if self.lists.len() == 1 {
            let mut result = self.lists.pop().unwrap();
            if self.sorted {
                result.sort_by(|a, b| a.rank.cmp(&b.rank).reverse());
            }
            return result;
        }

        let mut merged: Vec<Result> = Vec::new();
        for list in &self.lists {
            merged.extend(list.iter().cloned());
        }

        if self.sorted {
            merged.sort_by(|a, b| a.rank.cmp(&b.rank).reverse());
        }

        merged
    }

    pub fn len(&self) -> usize {
        self.lists.iter().map(|l| l.len()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.lists.iter().all(|l| l.is_empty())
    }

    pub fn cacheable(&self) -> bool {
        self.final_
    }

    pub fn is_final(&self) -> bool {
        self.final_
    }

    pub fn get(&self, index: usize) -> Option<&Result> {
        let mut offset = 0;
        for list in &self.lists {
            if index < offset + list.len() {
                return list.get(index - offset);
            }
            offset += list.len();
        }
        None
    }
}

impl Default for Merger {
    fn default() -> Self {
        Self {
            lists: Vec::new(),
            sorted: true,
            final_: false,
            ascending: true,
        }
    }
}

pub struct PassMerger {
    items: Vec<Result>,
}

impl PassMerger {
    pub fn new(items: Vec<Result>) -> Self {
        Self { items }
    }

    pub fn get(&self, index: usize) -> Option<&Result> {
        self.items.get(index)
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}
