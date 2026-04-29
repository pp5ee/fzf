use crate::item::AnsiColor;

#[derive(Debug, Clone)]
pub struct AnsiState {
    pub fg: i32,
    pub bg: i32,
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underline: bool,
    pub blink: bool,
    pub reverse: bool,
    pub hidden: bool,
    pub strike: bool,
}

impl AnsiState {
    pub fn new() -> Self {
        Self {
            fg: -1,
            bg: -1,
            bold: false,
            dim: false,
            italic: false,
            underline: false,
            blink: false,
            reverse: false,
            hidden: false,
            strike: false,
        }
    }

    pub fn reset(&mut self) {
        self.fg = -1;
        self.bg = -1;
        self.bold = false;
        self.dim = false;
        self.italic = false;
        self.underline = false;
        self.blink = false;
        self.reverse = false;
        self.hidden = false;
        self.strike = false;
    }
}

impl Default for AnsiState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn extract_color(text: &str, state: Option<&AnsiState>, _end_state: Option<&mut AnsiState>) -> (String, Vec<(usize, AnsiColor)>, Option<AnsiState>) {
    let mut result = String::new();
    let mut colors = Vec::new();
    let mut current_state = state.map(|s| s.clone()).unwrap_or_default();
    let mut last_pos = 0;

    for c in text.chars() {
        if c == '\u{1b}' {
            // Simplified: skip escape sequences
            continue;
        }
        result.push(c);
        last_pos += 1;
    }

    (result, colors, Some(current_state))
}
