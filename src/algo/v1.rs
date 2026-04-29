use super::{normalize::normalize_rune, Case, CharClass, MatchResult, SCORE_GAP_EXTENSION, SCORE_GAP_START, SCORE_MATCH, BONUS_FIRST_CHAR_MULTIPLIER, BONUS_CONSECUTIVE, Scheme};

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

    let first_pat_char = match case {
        Case::Respect => pattern[0],
        _ => to_lower(pattern[0]),
    };

    let mut pidx = 0;
    let mut sidx = 0;
    let mut eidx = 0;
    let mut started = false;

    let scheme = Scheme::current();
    let mut prev_class = if scheme == Scheme::Path {
        CharClass::Delimiter
    } else {
        CharClass::White
    };

    while sidx < text_len {
        let c = text[sidx];
        let class = CharClass::from_char(c);
        let c_lower = to_lower(c);

        let pchar = if case == Case::Respect {
            pattern[pidx]
        } else {
            to_lower(pattern[pidx])
        };

        if c_lower == pchar {
            if !started {
                if pidx == 0 {
                    let bonus = get_first_char_bonus(prev_class, class, c, c_lower);
                    if bonus < 0 && sidx > 0 {
                        sidx += 1;
                        prev_class = class;
                        continue;
                    }
                }
                started = true;
            }

            pidx += 1;
            if pidx == pat_len {
                eidx = sidx + 1;
                break;
            }
        } else if started {
            if forward {
                return None;
            }
        }

        sidx += 1;
        prev_class = class;
    }

    if pidx < pat_len {
        return None;
    }

    let (best_start, best_end) = if forward {
        (0, eidx)
    } else {
        optimize_backward(pattern, text, case, normalize, eidx)
    };

    let score = calculate_score(pattern, text, case, normalize, best_start, best_end);

    Some(MatchResult {
        start: best_start,
        end: best_end,
        score,
    })
}

fn get_first_char_bonus(
    prev_class: CharClass,
    class: CharClass,
    c: char,
    c_lower: char,
) -> i32 {
    let scheme = Scheme::current();
    let base_bonus = super::get_bonus_matrix(prev_class, class);

    if c == c_lower {
        base_bonus * BONUS_FIRST_CHAR_MULTIPLIER
    } else {
        let uppercase_bonus = if class == CharClass::Upper {
            BONUS_FIRST_CHAR_MULTIPLIER
        } else {
            0
        };
        base_bonus * BONUS_FIRST_CHAR_MULTIPLIER + uppercase_bonus
    }
}

fn optimize_backward(
    pattern: &[char],
    text: &[char],
    case: Case,
    normalize: bool,
    end: usize,
) -> (usize, usize) {
    let pat_len = pattern.len();
    let mut pidx = pat_len - 1;
    let mut sidx = end - 1;
    let mut best_start = sidx;
    let mut best_end = end;

    let to_lower = |c: char| {
        if normalize {
            normalize_rune(c)
        } else {
            c.to_lowercase().next().unwrap_or(c)
        }
    };

    loop {
        let pchar = if case == Case::Respect {
            pattern[pidx]
        } else {
            to_lower(pattern[pidx])
        };
        let c = text[sidx];
        let c_lower = to_lower(c);

        if c_lower == pchar {
            if pidx == 0 {
                best_start = sidx;
                break;
            }
            if pidx > 0 {
                pidx -= 1;
            }
        }

        if sidx > 0 {
            sidx -= 1;
        } else {
            break;
        }
    }

    (best_start, best_end)
}

fn calculate_score(
    pattern: &[char],
    text: &[char],
    case: Case,
    normalize: bool,
    start: usize,
    end: usize,
) -> i32 {
    let pat_len = pattern.len();
    let mut score = 0i32;
    let mut consecutive = 0i32;
    let mut prev_class = CharClass::White;

    let scheme = Scheme::current();
    if scheme == Scheme::Path {
        prev_class = CharClass::Delimiter;
    }

    let to_lower = |c: char| {
        if normalize {
            normalize_rune(c)
        } else {
            c.to_lowercase().next().unwrap_or(c)
        }
    };

    let mut pidx = 0;
    for (idx, &c) in text.iter().enumerate().take(end).skip(start) {
        let class = CharClass::from_char(c);
        let c_lower = to_lower(c);
        let pchar = if case == Case::Respect {
            if pidx < pat_len {
                pattern[pidx]
            } else {
                '\0'
            }
        } else if pidx < pat_len {
            to_lower(pattern[pidx])
        } else {
            '\0'
        };

        if pidx < pat_len && c_lower == pchar {
            let bonus = super::get_bonus_matrix(prev_class, class);
            score += SCORE_MATCH + bonus;

            if consecutive > 0 {
                score += BONUS_CONSECUTIVE;
            }
            consecutive += 1;

            if pidx == 0 {
                let first_bonus = if c == c_lower {
                    bonus * (BONUS_FIRST_CHAR_MULTIPLIER - 1)
                } else {
                    bonus * (BONUS_FIRST_CHAR_MULTIPLIER - 1) + if class == CharClass::Upper { 1 } else { 0 }
                };
                score += first_bonus;
            }

            pidx += 1;
        } else {
            if consecutive > 0 {
                score += SCORE_GAP_START + SCORE_GAP_EXTENSION * (consecutive - 1);
            } else {
                score += SCORE_GAP_EXTENSION;
            }
            consecutive = 0;
        }

        prev_class = class;
    }

    score
}
