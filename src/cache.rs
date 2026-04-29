use crate::algo::MatchResult;
use crate::pattern::Pattern;
use crate::util::chars::Chunk;
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct CacheEntry {
    pub result: MatchResult,
    pub pattern_revision: u32,
}

pub struct ChunkCache {
    cache: HashMap<u64, HashMap<usize, CacheEntry>>,
    revision: u32,
}

impl ChunkCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            revision: 0,
        }
    }

    pub fn get(&self, chunk: &Chunk, pattern: &Pattern) -> Option<MatchResult> {
        let chunk_key = chunk as *const _ as u64;
        let pattern_key = pattern.cache_key();

        let pattern_hash = {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            pattern_key.hash(&mut hasher);
            hasher.finish() as usize
        };

        self.cache
            .get(&chunk_key)
            .and_then(|inner| inner.get(&pattern_hash))
            .filter(|entry| entry.pattern_revision == self.revision)
            .map(|entry| entry.result.clone())
    }

    pub fn set(&mut self, chunk: &Chunk, pattern: &Pattern, result: MatchResult) {
        let chunk_key = chunk as *const _ as u64;
        let pattern_key = pattern.cache_key();

        let pattern_hash = {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            pattern_key.hash(&mut hasher);
            hasher.finish() as usize
        };

        let entry = CacheEntry {
            result,
            pattern_revision: self.revision,
        };

        self.cache
            .entry(chunk_key)
            .or_default()
            .insert(pattern_hash, entry);
    }

    pub fn clear(&mut self) {
        self.cache.clear();
        self.revision = self.revision.wrapping_add(1);
    }

    pub fn revision(&self) -> u32 {
        self.revision
    }
}

impl Default for ChunkCache {
    fn default() -> Self {
        Self::new()
    }
}
