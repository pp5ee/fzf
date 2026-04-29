use regex::Regex;
use std::sync::OnceLock;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub text: String,
    pub prefix: bool,
}

impl Token {
    pub fn new(text: String, prefix: bool) -> Self {
        Self { text, prefix }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delimiter {
    Regex,
    Str,
}

impl Default for Delimiter {
    fn default() -> Self {
        Self::Regex
    }
}

static AWK_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_awk_regex() -> &'static Regex {
    AWK_REGEX.get_or_init(|| {
        Regex::new(r"[ \t\n\r\x0b\x0c\x85\xa0]+").expect("Failed to compile AWK regex")
    })
}

pub fn tokenize(text: &str, delimiter: Delimiter) -> Vec<Token> {
    match delimiter {
        Delimiter::Regex => {
            let regex = get_awk_regex();
            let mut tokens = Vec::new();
            let mut last_end = 0;

            for mat in regex.find_iter(text) {
                if mat.start() > last_end {
                    tokens.push(Token::new(text[last_end..mat.start()].to_string(), false));
                }
                tokens.push(Token::new(mat.as_str().to_string(), true));
                last_end = mat.end();
            }

            if last_end < text.len() {
                tokens.push(Token::new(text[last_end..].to_string(), false));
            }

            tokens
        }
        Delimiter::Str => {
            text.split_whitespace()
                .map(|s| Token::new(s.to_string(), false))
                .collect()
        }
    }
}

pub fn strip_last_delimiter(text: String, delimiter: Delimiter) -> String {
    match delimiter {
        Delimiter::Regex => {
            let regex = get_awk_regex();
            regex.replace_all(&text, " ").trim_end().to_string()
        }
        Delimiter::Str => text.trim_end().to_string(),
    }
}
