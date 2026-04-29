use crate::algo::{Algo, Case, equal_match, exact_match_naive, fuzzy_match_v1, fuzzy_match_v2, MatchResult, prefix_match, suffix_match};
use regex::Regex;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TermType {
    Fuzzy,
    Exact,
    ExactBoundary,
    Prefix,
    Suffix,
    Equal,
}

#[derive(Debug, Clone)]
pub struct Term {
    pub typ: TermType,
    pub inv: bool,
    pub text: Vec<char>,
    pub case_sensitive: bool,
    pub normalize: bool,
}

impl Term {
    pub fn new(typ: TermType, inv: bool, text: Vec<char>, case_sensitive: bool, normalize: bool) -> Self {
        Self {
            typ,
            inv,
            text,
            case_sensitive,
            normalize,
        }
    }

    pub fn match_text(&self, text: &[char]) -> Option<MatchResult> {
        let case = if self.case_sensitive {
            Case::Respect
        } else {
            Case::Ignore
        };

        let result = match self.typ {
            TermType::Fuzzy => fuzzy_match_v2(&self.text, text, case, self.normalize, false),
            TermType::Exact => exact_match_naive(&self.text, text, case, self.normalize),
            TermType::ExactBoundary => exact_match_naive(&self.text, text, case, self.normalize),
            TermType::Prefix => prefix_match(&self.text, text, case, self.normalize),
            TermType::Suffix => suffix_match(&self.text, text, case, self.normalize),
            TermType::Equal => equal_match(&self.text, text, case, self.normalize),
        };

        result
    }
}

pub type TermSet = Vec<Term>;

#[derive(Debug, Clone)]
pub struct Pattern {
    pub fuzzy: bool,
    pub fuzzy_algo: Algo,
    pub extended: bool,
    pub case_sensitive: bool,
    pub normalize: bool,
    pub forward: bool,
    pub with_pos: bool,
    pub text: Vec<char>,
    pub term_sets: Vec<TermSet>,
    pub sortable: bool,
    pub cacheable: bool,
}

impl Pattern {
    pub fn new(query: &str, opts: &PatternOptions) -> Self {
        let text: Vec<char> = query.chars().collect();

        let mut pattern = Self {
            fuzzy: opts.fuzzy,
            fuzzy_algo: opts.fuzzy_algo,
            extended: opts.extended,
            case_sensitive: opts.case_sensitive,
            normalize: opts.normalize,
            forward: opts.forward,
            with_pos: opts.with_pos,
            text,
            term_sets: Vec::new(),
            sortable: true,
            cacheable: true,
        };

        if pattern.extended {
            pattern.parse_extended();
        } else {
            pattern.term_sets.push(vec![Term::new(
                if pattern.fuzzy { TermType::Fuzzy } else { TermType::Exact },
                false,
                pattern.text.clone(),
                pattern.case_sensitive,
                pattern.normalize,
            )]);
        }

        pattern
    }

    fn parse_extended(&mut self) {
        let query: String = self.text.iter().collect();
        let terms = split_terms(&query);

        let mut term_set: TermSet = Vec::new();

        for term_str in terms {
            if term_str.is_empty() {
                continue;
            }

            let (inv, rest) = if term_str.starts_with('!') {
                (true, &term_str[1..])
            } else {
                (false, &term_str[..])
            };

            if rest.is_empty() {
                continue;
            }

            let (typ, text) = if rest.starts_with("'") {
                (TermType::Exact, &rest[1..])
            } else if rest.starts_with("^") {
                (TermType::Prefix, &rest[1..])
            } else if rest.ends_with('$') {
                (TermType::Suffix, &rest[..rest.len().saturating_sub(1)])
            } else if rest.starts_with("=") {
                (TermType::Equal, &rest[1..])
            } else {
                (if self.fuzzy { TermType::Fuzzy } else { TermType::Exact }, rest)
            };

            if !text.is_empty() {
                term_set.push(Term::new(
                    typ,
                    inv,
                    text.chars().collect(),
                    self.case_sensitive,
                    self.normalize,
                ));
            }
        }

        if !term_set.is_empty() {
            self.term_sets.push(term_set);
        }
    }

    pub fn match_text(&self, text: &[char]) -> Option<MatchResult> {
        if self.term_sets.is_empty() {
            return Some(MatchResult::new());
        }

        let mut total_score = 0i32;
        let mut all_matched = true;
        let mut first_start = text.len();
        let mut last_end = 0;

        for term_set in &self.term_sets {
            let mut set_matched = false;
            let mut set_score = 0i32;

            for term in term_set {
                if let Some(result) = term.match_text(text) {
                    if !term.inv {
                        set_matched = true;
                        set_score = set_score.max(result.score);
                        first_start = first_start.min(result.start);
                        last_end = last_end.max(result.end);
                    }
                } else if term.inv {
                    set_matched = true;
                }
            }

            if !set_matched {
                all_matched = false;
                break;
            }

            total_score += set_score;
        }

        if all_matched {
            Some(MatchResult {
                start: first_start,
                end: last_end,
                score: total_score,
            })
        } else {
            None
        }
    }

    pub fn cache_key(&self) -> String {
        self.text.iter().collect()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PatternOptions {
    pub fuzzy: bool,
    pub fuzzy_algo: Algo,
    pub extended: bool,
    pub case_sensitive: bool,
    pub normalize: bool,
    pub forward: bool,
    pub with_pos: bool,
}

impl Default for PatternOptions {
    fn default() -> Self {
        Self {
            fuzzy: true,
            fuzzy_algo: Algo::V2,
            extended: true,
            case_sensitive: false,
            normalize: true,
            forward: true,
            with_pos: false,
        }
    }
}

fn split_terms(query: &str) -> Vec<&str> {
    query.split_whitespace().collect()
}
