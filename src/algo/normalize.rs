/// Normalize latin script letters by converting to lowercase and removing diacritics.
/// This is a simplified version for ASCII compatibility.
pub fn normalize_rune(c: char) -> char {
    // For now, implement basic case folding
    // Full normalization would require unicode-normalization crate
    c.to_lowercase().next().unwrap_or(c)
}

/// Convert text to lowercase for comparison
pub fn normalize_text(text: &str) -> String {
    text.to_lowercase()
}
