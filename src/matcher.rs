use crate::algo::{Algo, Case, fuzzy_match_v1, fuzzy_match_v2, MatchResult};
use crate::cache::ChunkCache;
use crate::item::Item;
use crate::merger::Merger;
use crate::pattern::Pattern;
use crate::result::{Rank, Result};
use crate::util::chars::Chunk;
use crate::util::eventbox::{EventBox, EventType, EventValue};
use crate::util::slab::IntSlab;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub const EVT_SEARCH_FINISHED: EventType = 1;
pub const EVT_SEARCH_PROGRESS: EventType = 2;
pub const EVT_SEARCH_NEW: EventType = 3;

pub struct MatchRequest {
    pub chunks: Vec<Arc<Chunk>>,
    pub pattern: Arc<Pattern>,
    pub final_: bool,
    pub sort: bool,
}

pub struct Matcher {
    cache: ChunkCache,
    event_box: EventBox,
    req_box: EventBox,
    partitions: usize,
    cancel_flag: Arc<AtomicBool>,
    revision: u32,
}

impl Matcher {
    pub fn new(
        cache: ChunkCache,
        event_box: EventBox,
        revision: u32,
        threads: Option<usize>,
    ) -> Self {
        let partitions = threads.unwrap_or_else(|| num_cpus::get());

        Self {
            cache,
            event_box,
            req_box: EventBox::new(),
            partitions,
            cancel_flag: Arc::new(AtomicBool::new(false)),
            revision,
        }
    }

    pub fn loop_matcher(&self) {
        loop {
            if let Some((event_type, value)) = self.req_box.try_recv() {
                match (event_type, value) {
                    (0, _) => break, // Quit signal
                    _ => {}
                }
            }

            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }

    pub fn match_chunk(
        &self,
        chunk: &Chunk,
        pattern: &Pattern,
    ) -> Vec<Result> {
        let mut results = Vec::new();

        for (idx, item) in chunk.items.iter().enumerate() {
            if self.cancel_flag.load(Ordering::Relaxed) {
                break;
            }

            let text = item.to_vec();

            let match_result = if pattern.fuzzy {
                match pattern.fuzzy_algo {
                    Algo::V1 => fuzzy_match_v1(
                        &pattern.text,
                        &text,
                        if pattern.case_sensitive { Case::Respect } else { Case::Ignore },
                        pattern.normalize,
                        pattern.forward,
                    ),
                    Algo::V2 => fuzzy_match_v2(
                        &pattern.text,
                        &text,
                        if pattern.case_sensitive { Case::Respect } else { Case::Ignore },
                        pattern.normalize,
                        pattern.forward,
                    ),
                }
            } else {
                pattern.match_text(&text)
            };

            if let Some(matched) = match_result {
                results.push(Result::new(
                    Arc::new(Item::new(item.clone())),
                    Rank::new(
                        matched.score,
                        item.index() as u32,
                        item.trim_length(),
                    ),
                    matched,
                ));
            }
        }

        results
    }

    pub fn match_chunks(
        &self,
        chunks: &[Arc<Chunk>],
        pattern: &Pattern,
    ) -> Vec<Result> {
        let mut all_results = Vec::new();

        for chunk in chunks {
            if self.cancel_flag.load(Ordering::Relaxed) {
                break;
            }

            let results = self.match_chunk(chunk, pattern);
            all_results.extend(results);
        }

        all_results
    }

    pub fn cancel(&self) {
        self.cancel_flag.store(true, Ordering::Relaxed);
    }

    pub fn reset_cancel(&self) {
        self.cancel_flag.store(false, Ordering::Relaxed);
    }
}

fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|p| p.get())
        .unwrap_or(1)
}
