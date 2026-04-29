pub mod v1;
pub mod v2;
pub mod normalize;

#[cfg(test)]
mod tests;

use std::sync::atomic::{AtomicUsize, Ordering};

pub const SCORE_MATCH: i32 = 16;
pub const SCORE_GAP_START: i32 = -3;
pub const SCORE_GAP_EXTENSION: i32 = -1;
pub const BONUS_BOUNDARY: i32 = SCORE_MATCH / 2;
pub const BONUS_NON_WORD: i32 = SCORE_MATCH / 2;
pub const BONUS_CAMEL123: i32 = BONUS_BOUNDARY + SCORE_GAP_EXTENSION;
pub const BONUS_CONSECUTIVE: i32 = -(SCORE_GAP_START + SCORE_GAP_EXTENSION);
pub const BONUS_FIRST_CHAR_MULTIPLIER: i32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MatchResult {
    pub start: usize,
    pub end: usize,
    pub score: i32,
}

impl MatchResult {
    pub const fn new() -> Self {
        Self {
            start: 0,
            end: 0,
            score: 0,
        }
    }

    pub const fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Algo {
    V1,
    V2,
}

impl Default for Algo {
    fn default() -> Self {
        Self::V2
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Case {
    Smart,
    Ignore,
    Respect,
}

impl Default for Case {
    fn default() -> Self {
        Self::Smart
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CharClass {
    White = 0,
    NonWord = 1,
    Delimiter = 2,
    Lower = 3,
    Upper = 4,
    Letter = 5,
    Number = 6,
}

impl CharClass {
    pub fn from_char(c: char) -> Self {
        match c {
            ' ' | '\t' | '\n' | '\r' | '\u{0b}' | '\u{0c}' | '\u{85}' | '\u{A0}' => Self::White,
            'a'..='z' => Self::Lower,
            'A'..='Z' => Self::Upper,
            '0'..='9' => Self::Number,
            '/' | ':' | ',' | ';' | '|' => Self::Delimiter,
            _ if c.is_alphabetic() => Self::Letter,
            _ => Self::NonWord,
        }
    }

    pub fn is_word(&self) -> bool {
        matches!(self, Self::Lower | Self::Upper | Self::Letter | Self::Number)
    }
}

static SCHEME: AtomicUsize = AtomicUsize::new(Scheme::Default as usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum Scheme {
    Default = 0,
    Path = 1,
    History = 2,
}

impl Scheme {
    pub fn init(scheme: &str) -> bool {
        let s = match scheme {
            "default" => Self::Default,
            "path" => Self::Path,
            "history" => Self::History,
            _ => return false,
        };
        SCHEME.store(s as usize, Ordering::Relaxed);
        true
    }

    pub fn current() -> Self {
        match SCHEME.load(Ordering::Relaxed) {
            1 => Self::Path,
            2 => Self::History,
            _ => Self::Default,
        }
    }

    pub fn bonus_boundary_white(&self) -> i32 {
        match self {
            Self::Default => BONUS_BOUNDARY + 2,
            _ => BONUS_BOUNDARY,
        }
    }

    pub fn bonus_boundary_delimiter(&self) -> i32 {
        match self {
            Self::Default => BONUS_BOUNDARY + 1,
            Self::Path => BONUS_BOUNDARY + 1,
            Self::History => BONUS_BOUNDARY,
        }
    }
}

pub fn fuzzy_match_v1(
    pattern: &[char],
    text: &[char],
    case: Case,
    normalize: bool,
    forward: bool,
) -> Option<MatchResult> {
    v1::fuzzy_match(pattern, text, case, normalize, forward)
}

pub fn fuzzy_match_v2(
    pattern: &[char],
    text: &[char],
    case: Case,
    normalize: bool,
    forward: bool,
) -> Option<MatchResult> {
    v2::fuzzy_match(pattern, text, case, normalize, forward)
}

pub fn exact_match_naive(
    pattern: &[char],
    text: &[char],
    case: Case,
    normalize: bool,
) -> Option<MatchResult> {
    if pattern.is_empty() {
        return Some(MatchResult::new());
    }

    let pat_len = pattern.len();
    let text_len = text.len();

    if pat_len > text_len {
        return None;
    }

    let to_lower = |c: char| {
        if normalize {
            normalize::normalize_rune(c)
        } else {
            c.to_lowercase().next().unwrap_or(c)
        }
    };

    let (pattern_chars, text_chars): (Vec<char>, Vec<char>) = match case {
        Case::Respect => (pattern.to_vec(), text.to_vec()),
        _ => (
            pattern.iter().copied().map(to_lower).collect(),
            text.iter().copied().map(to_lower).collect(),
        ),
    };

    for i in 0..=text_len - pat_len {
        if &text_chars[i..i + pat_len] == &pattern_chars[..] {
            return Some(MatchResult {
                start: i,
                end: i + pat_len,
                score: SCORE_MATCH * pat_len as i32,
            });
        }
    }

    None
}

pub fn prefix_match(
    pattern: &[char],
    text: &[char],
    case: Case,
    normalize: bool,
) -> Option<MatchResult> {
    if pattern.is_empty() {
        return Some(MatchResult::new());
    }

    let pat_len = pattern.len();
    let text_len = text.len();

    if pat_len > text_len {
        return None;
    }

    let to_lower = |c: char| {
        if normalize {
            normalize::normalize_rune(c)
        } else {
            c.to_lowercase().next().unwrap_or(c)
        }
    };

    let (pattern_chars, text_chars): (Vec<char>, Vec<char>) = match case {
        Case::Respect => (pattern.to_vec(), text.to_vec()),
        _ => (
            pattern.iter().copied().map(to_lower).collect(),
            text.iter().copied().map(to_lower).collect(),
        ),
    };

    if text_chars.starts_with(&pattern_chars) {
        return Some(MatchResult {
            start: 0,
            end: pat_len,
            score: SCORE_MATCH * pat_len as i32,
        });
    }

    None
}

pub fn suffix_match(
    pattern: &[char],
    text: &[char],
    case: Case,
    normalize: bool,
) -> Option<MatchResult> {
    if pattern.is_empty() {
        return Some(MatchResult::new());
    }

    let pat_len = pattern.len();
    let text_len = text.len();

    if pat_len > text_len {
        return None;
    }

    let to_lower = |c: char| {
        if normalize {
            normalize::normalize_rune(c)
        } else {
            c.to_lowercase().next().unwrap_or(c)
        }
    };

    let (pattern_chars, text_chars): (Vec<char>, Vec<char>) = match case {
        Case::Respect => (pattern.to_vec(), text.to_vec()),
        _ => (
            pattern.iter().copied().map(to_lower).collect(),
            text.iter().copied().map(to_lower).collect(),
        ),
    };

    if text_chars.ends_with(&pattern_chars) {
        return Some(MatchResult {
            start: text_len - pat_len,
            end: text_len,
            score: SCORE_MATCH * pat_len as i32,
        });
    }

    None
}

pub fn equal_match(
    pattern: &[char],
    text: &[char],
    case: Case,
    normalize: bool,
) -> Option<MatchResult> {
    if pattern.len() != text.len() {
        return None;
    }

    let to_lower = |c: char| {
        if normalize {
            normalize::normalize_rune(c)
        } else {
            c.to_lowercase().next().unwrap_or(c)
        }
    };

    let equal = match case {
        Case::Respect => pattern == text,
        _ => {
            let p: Vec<char> = pattern.iter().copied().map(to_lower).collect();
            let t: Vec<char> = text.iter().copied().map(to_lower).collect();
            p == t
        }
    };

    if equal {
        Some(MatchResult {
            start: 0,
            end: text.len(),
            score: SCORE_MATCH * text.len() as i32,
        })
    } else {
        None
    }
}

pub fn get_bonus_matrix(cc_prev: CharClass, cc_curr: CharClass) -> i32 {
    static BONUS_MATRIX: [[i32; 7]; 7] = [
        [BONUS_BOUNDARY + 2, BONUS_BOUNDARY, 0, BONUS_BOUNDARY, BONUS_BOUNDARY, BONUS_BOUNDARY, BONUS_BOUNDARY],
        [BONUS_BOUNDARY + 2, 0, 0, BONUS_BOUNDARY, BONUS_BOUNDARY, BONUS_BOUNDARY, 0],
        [BONUS_BOUNDARY + 2, BONUS_BOUNDARY, BONUS_BOUNDARY, BONUS_BOUNDARY, BONUS_BOUNDARY, BONUS_BOUNDARY, BONUS_BOUNDARY],
        [BONUS_BOUNDARY + 2, BONUS_BOUNDARY, 0, 0, 0, 0, BONUS_CAMEL123],
        [BONUS_BOUNDARY + 2, BONUS_BOUNDARY, 0, BONUS_BOUNDARY, BONUS_BOUNDARY, BONUS_BOUNDARY, BONUS_CAMEL123],
        [BONUS_BOUNDARY + 2, BONUS_BOUNDARY, 0, 0, 0, 0, BONUS_CAMEL123],
        [BONUS_BOUNDARY + 2, 0, 0, 0, 0, 0, 0],
    ];

    BONUS_MATRIX[cc_prev as usize][cc_curr as usize]
}
