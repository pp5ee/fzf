use crate::tokenizer::{tokenize, Token};
use crate::util::chars::Chars;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct AnsiOffset {
    pub offset: [u16; 2],
    pub color: AnsiColor,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct AnsiColor {
    pub fg: i32,
    pub bg: i32,
    pub attrs: u16,
}

impl AnsiColor {
    pub const fn new() -> Self {
        Self {
            fg: -1,
            bg: -1,
            attrs: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.fg < 0 && self.bg < 0 && self.attrs == 0
    }
}

#[derive(Debug, Clone)]
pub struct Transformed {
    pub revision: u32,
    pub tokens: Vec<Token>,
}

#[derive(Debug, Clone)]
pub struct Item {
    pub text: Chars,
    pub transformed: Option<Transformed>,
    pub orig_text: Option<Arc<Vec<u8>>>,
    pub colors: Option<Arc<Vec<AnsiOffset>>>,
}

impl Item {
    pub fn new(text: Chars) -> Self {
        Self {
            text,
            transformed: None,
            orig_text: None,
            colors: None,
        }
    }

    pub fn with_original(mut self, orig: Vec<u8>) -> Self {
        self.orig_text = Some(Arc::new(orig));
        self
    }

    pub fn with_colors(mut self, colors: Vec<AnsiOffset>) -> Self {
        self.colors = Some(Arc::new(colors));
        self
    }

    pub fn with_transformed(mut self, revision: u32, tokens: Vec<Token>) -> Self {
        self.transformed = Some(Transformed { revision, tokens });
        self
    }

    pub fn index(&self) -> i32 {
        self.text.index()
    }

    pub fn trim_length(&self) -> u16 {
        self.text.trim_length()
    }

    pub fn as_string(&self, strip_ansi: bool) -> String {
        if let Some(orig) = &self.orig_text {
            if strip_ansi {
                strip_ansi_codes(&String::from_utf8_lossy(orig))
            } else {
                String::from_utf8_lossy(orig).to_string()
            }
        } else {
            self.text.as_str().to_string()
        }
    }

    pub fn text(&self) -> &Chars {
        &self.text
    }
}

fn strip_ansi_codes(s: &str) -> String {
    // Simple ANSI stripping - a full implementation would be more complex
    let mut result = String::new();
    let mut in_escape = false;

    for c in s.chars() {
        if in_escape {
            if c.is_ascii_alphabetic() || c == '~' {
                in_escape = false;
            }
        } else if c == '\x1b' {
            in_escape = true;
        } else {
            result.push(c);
        }
    }

    result
}
