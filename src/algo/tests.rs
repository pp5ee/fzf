#[cfg(test)]
mod tests {
    use super::super::{fuzzy_match_v1, fuzzy_match_v2, Case, MatchResult};

    fn to_chars(s: &str) -> Vec<char> {
        s.chars().collect()
    }

    #[test]
    fn test_fuzzy_match_v1_basic() {
        let pattern = to_chars("ab");
        let text = to_chars("about banana");

        let result = fuzzy_match_v1(&pattern, &text, Case::Ignore, true, true);
        assert!(result.is_some());

        let m = result.unwrap();
        assert!(m.score > 0);
        assert_eq!(m.start, 0);
    }

    #[test]
    fn test_fuzzy_match_v1_exact() {
        let pattern = to_chars("abc");
        let text = to_chars("abc");

        let result = fuzzy_match_v1(&pattern, &text, Case::Ignore, true, true);
        assert!(result.is_some());

        let m = result.unwrap();
        assert!(m.score > 0);
        assert_eq!(m.start, 0);
        assert_eq!(m.end, 3);
    }

    #[test]
    fn test_fuzzy_match_v1_no_match() {
        let pattern = to_chars("xyz");
        let text = to_chars("abc");

        let result = fuzzy_match_v1(&pattern, &text, Case::Ignore, true, true);
        assert!(result.is_none());
    }

    #[test]
    fn test_fuzzy_match_v1_case_sensitive() {
        let pattern = to_chars("ABC");
        let text = to_chars("abc");

        let result = fuzzy_match_v1(&pattern, &text, Case::Respect, true, true);
        assert!(result.is_none());

        let result = fuzzy_match_v1(&pattern, &text, Case::Ignore, true, true);
        assert!(result.is_some());
    }

    #[test]
    fn test_fuzzy_match_v1_empty_pattern() {
        let pattern = to_chars("");
        let text = to_chars("abc");

        let result = fuzzy_match_v1(&pattern, &text, Case::Ignore, true, true);
        assert!(result.is_some());

        let m = result.unwrap();
        assert_eq!(m.score, 0);
    }

    #[test]
    fn test_fuzzy_match_v2_basic() {
        let pattern = to_chars("ab");
        let text = to_chars("apple banana");

        let result = fuzzy_match_v2(&pattern, &text, Case::Ignore, true, true);
        assert!(result.is_some());

        let m = result.unwrap();
        assert!(m.score > 0);
    }

    #[test]
    fn test_fuzzy_match_v2_vs_v1() {
        // Both V1 and V2 should find matches
        let pattern = to_chars("abc");
        let text = to_chars("abc");

        let v1_result = fuzzy_match_v1(&pattern, &text, Case::Ignore, true, true);
        let v2_result = fuzzy_match_v2(&pattern, &text, Case::Ignore, true, true);

        assert!(v1_result.is_some());
        assert!(v2_result.is_some());

        // V2 should have equal or better score than V1
        assert!(v2_result.unwrap().score >= v1_result.unwrap().score);
    }

    #[test]
    fn test_fuzzy_match_v2_exact() {
        let pattern = to_chars("abc");
        let text = to_chars("abc");

        let result = fuzzy_match_v2(&pattern, &text, Case::Ignore, true, true);
        assert!(result.is_some());

        let m = result.unwrap();
        assert!(m.score > 0);
        assert_eq!(m.start, 0);
        assert_eq!(m.end, 3);
    }

    #[test]
    fn test_word_boundary_bonus() {
        // Test that matches at word boundaries get higher scores
        let pattern = to_chars("fb");
        let text1 = to_chars("foobar");      // f-o-o-b (no word boundary at b)
        let text2 = to_chars("foo-bar");     // f-o-o-|-b (word boundary at b)

        let result1 = fuzzy_match_v2(&pattern, &text1, Case::Ignore, true, true);
        let result2 = fuzzy_match_v2(&pattern, &text2, Case::Ignore, true, true);

        assert!(result1.is_some());
        assert!(result2.is_some());

        // The dash creates a word boundary, giving text2 higher score
        assert!(result2.unwrap().score > result1.unwrap().score);
    }

    #[test]
    fn test_camelcase_bonus() {
        // Test that camelCase matches get bonuses
        let pattern = to_chars("fb");
        let text = to_chars("FooBar");  // F and B are uppercase

        let result = fuzzy_match_v2(&pattern, &text, Case::Ignore, true, true);
        assert!(result.is_some());
        assert!(result.unwrap().score > 0);
    }

    #[test]
    fn test_gap_penalty() {
        // Longer gaps should reduce score
        let pattern = to_chars("az");
        let text1 = to_chars("az");           // No gap
        let text2 = to_chars("a........z");   // Long gap

        let result1 = fuzzy_match_v2(&pattern, &text1, Case::Ignore, true, true);
        let result2 = fuzzy_match_v2(&pattern, &text2, Case::Ignore, true, true);

        assert!(result1.is_some());
        assert!(result2.is_some());

        // Shorter match should have higher score
        assert!(result1.unwrap().score > result2.unwrap().score);
    }

    #[test]
    fn test_consecutive_bonus() {
        // Consecutive matches should get bonus
        let pattern = to_chars("abc");
        let text1 = to_chars("abc");      // Consecutive
        let text2 = to_chars("aXbXc");    // Non-consecutive

        let result1 = fuzzy_match_v2(&pattern, &text1, Case::Ignore, true, true);
        let result2 = fuzzy_match_v2(&pattern, &text2, Case::Ignore, true, true);

        assert!(result1.is_some());
        assert!(result2.is_some());

        // Consecutive should score higher
        assert!(result1.unwrap().score > result2.unwrap().score);
    }

    #[test]
    fn test_first_char_multiplier() {
        // First character match at boundary should get bonus
        let pattern = to_chars("abc");
        let text = to_chars("abc xyz");

        let result = fuzzy_match_v2(&pattern, &text, Case::Ignore, true, true);
        assert!(result.is_some());

        let m = result.unwrap();
        assert_eq!(m.start, 0); // Should match at beginning
    }

    #[test]
    fn test_unicode_handling() {
        let pattern = to_chars("日本");
        let text = to_chars("日本語テキスト");

        let result = fuzzy_match_v1(&pattern, &text, Case::Ignore, true, true);
        assert!(result.is_some());

        let m = result.unwrap();
        assert!(m.score > 0);
    }

    #[test]
    fn test_long_pattern_no_match() {
        let pattern = to_chars("thisisaverylongpatternthatwontmatch");
        let text = to_chars("short");

        let result = fuzzy_match_v1(&pattern, &text, Case::Ignore, true, true);
        assert!(result.is_none());
    }
}
