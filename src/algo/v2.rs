use super::{normalize::normalize_rune, Case, CharClass, MatchResult, SCORE_GAP_EXTENSION, SCORE_GAP_START, SCORE_MATCH, BONUS_FIRST_CHAR_MULTIPLIER, BONUS_CONSECUTIVE, Scheme};

#[derive(Debug, Clone, Copy, Default)]
struct ScoreMatrix {
    score: i32,
    consecutive: i32,
}

pub fn fuzzy_match(
    pattern: &[char],
    text: &[char],
    case: Case,
    normalize: bool,
    forward: bool,
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
            normalize_rune(c)
        } else {
            c.to_lowercase().next().unwrap_or(c)
        }
    };

    let pattern_chars: Vec<char> = match case {
        Case::Respect => pattern.to_vec(),
        _ => pattern.iter().copied().map(to_lower).collect(),
    };

    let text_chars: Vec<char> = text.iter().copied().map(to_lower).collect();

    let mut m = vec![vec![ScoreMatrix::default(); text_len]; pat_len];

    let scheme = Scheme::current();
    let mut prev_class = if scheme == Scheme::Path {
        CharClass::Delimiter
    } else {
        CharClass::White
    };

    for (j, &tc) in text_chars.iter().enumerate() {
        let class = CharClass::from_char(text[j]);
        let pc = pattern_chars[0];

        if tc == pc {
            let bonus = super::get_bonus_matrix(prev_class, class);
            let first_bonus = if pattern[0] == text[j] || text[j].to_lowercase().next() != Some(text[j]) {
                0
            } else {
                1
            };
            m[0][j].score = SCORE_MATCH + bonus + first_bonus * (BONUS_FIRST_CHAR_MULTIPLIER - 1);
            m[0][j].consecutive = 1;
        }

        prev_class = class;
    }

    for i in 1..pat_len {
        let pc = pattern_chars[i];
        prev_class = if scheme == Scheme::Path {
            CharClass::Delimiter
        } else {
            CharClass::White
        };

        for j in i..text_len {
            let class = CharClass::from_char(text[j]);
            let tc = text_chars[j];

            if tc == pc {
                let bonus = super::get_bonus_matrix(prev_class, class);
                let diag = if j > 0 {
                    m[i - 1][j - 1]
                } else {
                    ScoreMatrix::default()
                };

                let match_score = if diag.consecutive > 0 {
                    diag.score + SCORE_MATCH + bonus + BONUS_CONSECUTIVE
                } else {
                    diag.score + SCORE_MATCH + bonus
                };

                if i == 0 {
                    let first_bonus = if pattern[0] == text[j] || text[j].to_lowercase().next() != Some(text[j]) {
                        0
                    } else {
                        1
                    };
                    m[i][j].score = SCORE_MATCH + bonus + first_bonus * (BONUS_FIRST_CHAR_MULTIPLIER - 1);
                } else {
                    m[i][j].score = match_score;
                }

                m[i][j].consecutive = diag.consecutive + 1;
            } else {
                m[i][j].score = i32::MIN / 2;
                m[i][j].consecutive = 0;
            }

            prev_class = class;
        }
    }

    let mut best_score = i32::MIN;
    let mut best_end = 0;

    for j in (pat_len - 1)..text_len {
        if m[pat_len - 1][j].score > best_score {
            best_score = m[pat_len - 1][j].score;
            best_end = j + 1;
        }
    }

    if best_score == i32::MIN {
        return None;
    }

    let best_start = if forward {
        0
    } else {
        backtrack_start(&m, pat_len, text_len)
    };

    Some(MatchResult {
        start: best_start,
        end: best_end,
        score: best_score,
    })
}

fn backtrack_start(m: &[Vec<ScoreMatrix>], pat_len: usize, text_len: usize) -> usize {
    let mut i = pat_len - 1;
    let mut j = text_len - 1;
    let mut best_start = j;

    while i > 0 {
        if m[i][j].consecutive > 0 && j > 0 && m[i - 1][j - 1].consecutive > 0 {
            j -= 1;
            i -= 1;
        } else if m[i][j].consecutive > 0 {
            best_start = j;
            j -= 1;
        } else if j > 0 {
            j -= 1;
        } else {
            break;
        }
    }

    best_start = best_start.saturating_sub(pat_len - 1);

    while best_start < text_len && m[0][best_start].consecutive == 0 {
        best_start += 1;
    }

    best_start
}
