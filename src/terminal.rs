use crate::item::Item;
use crate::options::{BorderStyle, Layout, Options};
use crate::pattern::{Pattern, PatternOptions};
use crate::reader::Reader;
use crate::result::Result as FzfResult;
use crate::util::chars::ChunkList;
use crate::util::eventbox::EventBox;
use crate::util::Executor;
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CrosstermEvent, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, Clear, ClearType},
    cursor::{Show, Hide},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout as RatatuiLayout, Margin, Rect, Position},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear as ClearWidget, List, ListItem, ListState, Paragraph, Wrap, BorderType},
    text::{Line, Span, Text},
    Frame, Terminal as RatatuiTerminal,
};
use std::io::{self, Write};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use rayon::prelude::*;

pub struct Terminal {
    options: Options,
    query: String,
    cursor_pos: usize,
    items: Vec<Arc<Item>>,
    filtered_items: Vec<FzfResult>,
    selection: ListState,
    reader: Option<Reader>,
    chunk_list: Arc<std::sync::Mutex<ChunkList>>,
    event_box: EventBox,
    running: Arc<AtomicBool>,
    multi_selection: Vec<Arc<Item>>,
    last_search: Instant,
    search_debounce: Duration,
    preview_visible: bool,
    preview_output: String,
    preview_scroll: usize,
    use_parallel: bool,
    height_mode: HeightMode,
    terminal_size: (u16, u16),
    scroll_offset: usize,
    info_style: InfoStyle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeightMode {
    Fullscreen,
    Fixed(u16),
    Percentage(u16),
    Auto(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InfoStyle {
    Default,
    Right,
    Hidden,
    Inline,
    InlineRight,
}

impl Terminal {
    pub fn new(options: Options) -> Self {
        let event_box = EventBox::new();
        let executor = Executor::new();
        let chunk_list = Arc::new(std::sync::Mutex::new(ChunkList::new(1000)));

        let reader = Reader::new(
            event_box.clone(),
            executor,
            options.read0,
        );

        let preview_visible = options.preview.is_some();

        let height_mode = if let Some(ref height_str) = options.height {
            Self::parse_height(height_str)
        } else {
            HeightMode::Fullscreen
        };

        let info_style = match options.info {
            crate::options::InfoStyle::Default => InfoStyle::Default,
            crate::options::InfoStyle::Right => InfoStyle::Right,
            crate::options::InfoStyle::Hidden => InfoStyle::Hidden,
            crate::options::InfoStyle::Inline => InfoStyle::Inline,
            crate::options::InfoStyle::InlineRight => InfoStyle::InlineRight,
        };

        let query = options.query.clone().unwrap_or_default();
        let cursor_pos = options.query.as_ref().map(|q| q.len()).unwrap_or(0);

        Self {
            options,
            query,
            cursor_pos,
            items: Vec::new(),
            filtered_items: Vec::new(),
            selection: ListState::default(),
            reader: Some(reader),
            chunk_list,
            event_box,
            running: Arc::new(AtomicBool::new(true)),
            multi_selection: Vec::new(),
            last_search: Instant::now(),
            search_debounce: Duration::from_millis(50),
            preview_visible,
            preview_output: String::new(),
            preview_scroll: 0,
            use_parallel: true,
            height_mode,
            terminal_size: (0, 0),
            scroll_offset: 0,
            info_style,
        }
    }

    fn parse_height(height_str: &str) -> HeightMode {
        let trimmed = height_str.trim();

        if trimmed.starts_with('~') {
            let num = trimmed[1..].trim_end_matches('%').parse::<u16>().unwrap_or(50);
            HeightMode::Auto(num)
        } else if trimmed.ends_with('%') {
            let num = trimmed[..trimmed.len()-1].parse::<u16>().unwrap_or(50);
            HeightMode::Percentage(num)
        } else if let Ok(num) = trimmed.parse::<u16>() {
            if trimmed.starts_with('-') {
                HeightMode::Auto(num)
            } else {
                HeightMode::Fixed(num)
            }
        } else {
            HeightMode::Fullscreen
        }
    }

    pub fn run(&mut self) -> Result<i32> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, Hide)?;

        let use_alternate = self.height_mode == HeightMode::Fullscreen;

        if use_alternate {
            execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        } else {
            execute!(stdout, Clear(ClearType::All))?;
        }

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = RatatuiTerminal::new(backend)?;

        let size = terminal.size()?;
        self.terminal_size = (size.width, size.height);

        let reader = self.reader.take().unwrap();
        std::thread::spawn(move || {
            let _ = reader.read_stdin();
        });

        self.load_items();

        if !self.query.is_empty() {
            self.perform_parallel_search();
        }

        let result = self.run_event_loop(&mut terminal);

        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            Show
        )?;

        if use_alternate {
            execute!(
                terminal.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            )?;
        }

        if result.is_ok() {
            self.output_selections()?;
        }

        result
    }

    fn run_event_loop<B: Backend>(&mut self, terminal: &mut RatatuiTerminal<B>) -> Result<i32> {
        let mut last_tick = Instant::now();
        let tick_rate = Duration::from_millis(100);

        while self.running.load(Ordering::Relaxed) {
            terminal.draw(|f| self.draw(f))?;

            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if crossterm::event::poll(timeout)? {
                if let Ok(event) = event::read() {
                    match event {
                        CrosstermEvent::Key(key) => {
                            if key.kind == KeyEventKind::Press {
                                self.handle_key_event(key.code, key.modifiers)?;
                            }
                        }
                        CrosstermEvent::Resize(_cols, _rows) => {
                            // Handle terminal resize
                            if let Ok(size) = terminal.size() {
                                self.terminal_size = (size.width, size.height);
                            }
                        }
                        _ => {}
                    }
                }
            }

            if last_tick.elapsed() >= tick_rate {
                self.load_items();
                self.perform_parallel_search();
                if self.preview_visible {
                    self.update_preview();
                }
                last_tick = Instant::now();
            }
        }

        Ok(0)
    }

    fn load_items(&mut self) {
        let list = match self.chunk_list.lock() {
            Ok(guard) => guard,
            Err(_) => return,
        };

        let start_index = self.items.len();

        for chunk in list.chunks() {
            for chars in &chunk.items {
                if chars.index() >= start_index as i32 {
                    self.items.push(Arc::new(Item::new(chars.clone())));
                }
            }
        }

        if start_index < self.items.len() && self.query.is_empty() {
            self.filtered_items = self.items.iter()
                .map(|item| FzfResult::new(
                    Arc::clone(item),
                    crate::result::Rank::new(0, item.index() as u32, item.trim_length()),
                    crate::algo::MatchResult::new(),
                ))
                .collect();

            if !self.options.tac {
                self.filtered_items.sort_by_key(|r| r.item.index());
            } else {
                self.filtered_items.sort_by_key(|r| std::cmp::Reverse(r.item.index()));
            }
        }
    }

    fn perform_parallel_search(&mut self) {
        if self.query.is_empty() {
            return;
        }

        if self.last_search.elapsed() < self.search_debounce {
            return;
        }

        let pattern_opts = PatternOptions {
            fuzzy: self.options.fuzzy,
            fuzzy_algo: match self.options.algo {
                crate::options::Algo::V1 => crate::algo::Algo::V1,
                crate::options::Algo::V2 => crate::algo::Algo::V2,
            },
            extended: self.options.extended,
            case_sensitive: self.options.case_sensitive == Some(true),
            normalize: self.options.normalize,
            forward: true,
            with_pos: false,
        };

        let pattern = Arc::new(Pattern::new(&self.query, &pattern_opts));

        if self.use_parallel && self.items.len() > 1000 {
            self.filtered_items = self.perform_parallel_matching(pattern);
        } else {
            self.filtered_items = self.perform_sequential_matching(&pattern);
        }

        self.last_search = Instant::now();

        if let Some(selected) = self.selection.selected() {
            if selected >= self.filtered_items.len() {
                self.selection.select(if self.filtered_items.is_empty() {
                    None
                } else {
                    Some(0)
                });
            }
        } else if !self.filtered_items.is_empty() {
            self.selection.select(Some(0));
        }
    }

    fn perform_parallel_matching(&self, pattern: Arc<Pattern>) -> Vec<FzfResult> {
        let items: Vec<_> = self.items.iter().map(|i| Arc::clone(i)).collect();

        let mut results: Vec<FzfResult> = items
            .par_iter()
            .filter_map(|item| {
                let text = item.text().to_vec();
                pattern.match_text(&text).map(|m| {
                    FzfResult::new(
                        Arc::clone(item),
                        crate::result::Rank::new(m.score, item.index() as u32, item.trim_length()),
                        m,
                    )
                })
            })
            .collect();

        if !self.options.no_sort {
            results.par_sort_by(|a, b| {
                b.rank.score.cmp(&a.rank.score)
                    .then_with(|| a.rank.length.cmp(&b.rank.length))
                    .then_with(|| b.rank.index.cmp(&a.rank.index))
            });
        }

        results
    }

    fn perform_sequential_matching(&self, pattern: &Pattern) -> Vec<FzfResult> {
        let mut results: Vec<FzfResult> = self.items
            .iter()
            .filter_map(|item| {
                let text = item.text().to_vec();
                pattern.match_text(&text).map(|m| {
                    FzfResult::new(
                        Arc::clone(item),
                        crate::result::Rank::new(m.score, item.index() as u32, item.trim_length()),
                        m,
                    )
                })
            })
            .collect();

        if !self.options.no_sort {
            results.sort_by(|a, b| {
                b.rank.score.cmp(&a.rank.score)
                    .then_with(|| a.rank.length.cmp(&b.rank.length))
                    .then_with(|| b.rank.index.cmp(&a.rank.index))
            });
        }

        results
    }

    fn update_preview(&mut self) {
        if let Some(preview_cmd) = &self.options.preview {
            if let Some(selected) = self.selection.selected() {
                if let Some(result) = self.filtered_items.get(selected) {
                    let item_text = result.item.as_string(true);
                    let cmd = preview_cmd.replace("{}", &item_text);
                    self.preview_output = self.execute_preview(&cmd);
                }
            }
        }
    }

    fn execute_preview(&self, cmd: &str) -> String {
        let output = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(["/C", cmd])
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .output()
        } else {
            Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .output()
        };

        match output {
            Ok(output) => {
                String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .take(100)
                    .collect::<Vec<_>>()
                    .join("\n")
            }
            Err(_) => String::from("[preview error]"),
        }
    }

    fn handle_key_event(&mut self, code: KeyCode, modifiers: KeyModifiers) -> Result<()> {
        match code {
            KeyCode::Char(c) => {
                if modifiers.contains(KeyModifiers::CONTROL) {
                    match c {
                        'c' => self.running.store(false, Ordering::Relaxed),
                        'u' => {
                            self.query.clear();
                            self.cursor_pos = 0;
                        }
                        'w' => self.delete_word(),
                        'a' => self.cursor_pos = 0,
                        'e' => self.cursor_pos = self.query.len(),
                        'n' => self.move_cursor(1),
                        'p' => self.move_cursor(-1),
                        'f' => self.move_cursor(1),
                        'b' => self.move_cursor(-1),
                        'd' => {
                            if self.cursor_pos < self.query.len() {
                                self.query.remove(self.cursor_pos);
                            }
                        }
                        'k' => {
                            self.query.truncate(self.cursor_pos);
                        }
                        'g' => self.running.store(false, Ordering::Relaxed),
                        'j' => self.move_selection(1),
                        'h' => self.move_selection(-1),
                        'l' => self.move_selection(1),
                        'm' => self.accept_selection()?,
                        '/' => self.toggle_preview(),
                        _ => {}
                    }
                } else if modifiers.contains(KeyModifiers::ALT) {
                    match c {
                        'b' => self.move_word(-1),
                        'f' => self.move_word(1),
                        'd' => self.delete_word_forward(),
                        _ => {}
                    }
                } else {
                    self.query.insert(self.cursor_pos, c);
                    self.cursor_pos += 1;
                    self.last_search = Instant::now() - self.search_debounce;
                }
            }
            KeyCode::Backspace => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                    self.query.remove(self.cursor_pos);
                    self.last_search = Instant::now() - self.search_debounce;
                }
            }
            KeyCode::Delete => {
                if self.cursor_pos < self.query.len() {
                    self.query.remove(self.cursor_pos);
                    self.last_search = Instant::now() - self.search_debounce;
                }
            }
            KeyCode::Left => {
                if modifiers.contains(KeyModifiers::CONTROL) {
                    self.move_word(-1);
                } else {
                    self.move_cursor(-1);
                }
            }
            KeyCode::Right => {
                if modifiers.contains(KeyModifiers::CONTROL) {
                    self.move_word(1);
                } else {
                    self.move_cursor(1);
                }
            }
            KeyCode::Up => {
                if modifiers.contains(KeyModifiers::CONTROL) {
                    self.scroll_preview(-1);
                } else {
                    self.move_selection(-1);
                }
            }
            KeyCode::Down => {
                if modifiers.contains(KeyModifiers::CONTROL) {
                    self.scroll_preview(1);
                } else {
                    self.move_selection(1);
                }
            }
            KeyCode::Home => self.cursor_pos = 0,
            KeyCode::End => self.cursor_pos = self.query.len(),
            KeyCode::PageUp => self.move_selection(-10),
            KeyCode::PageDown => self.move_selection(10),
            KeyCode::Enter => self.accept_selection()?,
            KeyCode::Tab => {
                if modifiers.contains(KeyModifiers::SHIFT) {
                    self.move_selection(-1);
                } else {
                    if self.options.multi.is_some() {
                        self.toggle_selection();
                    } else {
                        self.move_selection(1);
                    }
                }
            }
            KeyCode::Esc => self.running.store(false, Ordering::Relaxed),
            KeyCode::F(n) => {
                if n == 1 {
                    self.running.store(false, Ordering::Relaxed);
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn move_cursor(&mut self, delta: i32) {
        let new_pos = self.cursor_pos as i32 + delta;
        self.cursor_pos = new_pos.clamp(0, self.query.len() as i32) as usize;
    }

    fn move_word(&mut self, direction: i32) {
        if direction > 0 {
            while self.cursor_pos < self.query.len() &&
                  self.query.chars().nth(self.cursor_pos).map_or(false, |c| c.is_alphanumeric()) {
                self.cursor_pos += 1;
            }
            while self.cursor_pos < self.query.len() &&
                  self.query.chars().nth(self.cursor_pos).map_or(false, |c| !c.is_alphanumeric()) {
                self.cursor_pos += 1;
            }
        } else {
            while self.cursor_pos > 0 &&
                  self.query.chars().nth(self.cursor_pos - 1).map_or(false, |c| !c.is_alphanumeric()) {
                self.cursor_pos -= 1;
            }
            while self.cursor_pos > 0 &&
                  self.query.chars().nth(self.cursor_pos - 1).map_or(false, |c| c.is_alphanumeric()) {
                self.cursor_pos -= 1;
            }
        }
    }

    fn delete_word(&mut self) {
        if self.cursor_pos == 0 {
            return;
        }
        let mut pos = self.cursor_pos;
        while pos > 0 && self.query.chars().nth(pos - 1).map_or(false, |c| c.is_whitespace()) {
            pos -= 1;
        }
        while pos > 0 && self.query.chars().nth(pos - 1).map_or(false, |c| !c.is_whitespace()) {
            pos -= 1;
        }
        self.query.drain(pos..self.cursor_pos);
        self.cursor_pos = pos;
    }

    fn delete_word_forward(&mut self) {
        let old_pos = self.cursor_pos;
        self.move_word(1);
        self.query.drain(old_pos..self.cursor_pos);
        self.cursor_pos = old_pos;
    }

    fn move_selection(&mut self, delta: i32) {
        let current = self.selection.selected().unwrap_or(0);
        let new_pos = current as i32 + delta;
        let max = self.filtered_items.len().saturating_sub(1) as i32;

        if self.options.cycle {
            let wrapped = ((new_pos % (max + 1)) + (max + 1)) % (max + 1);
            self.selection.select(Some(wrapped as usize));
        } else {
            self.selection.select(Some(new_pos.clamp(0, max) as usize));
        }
    }

    fn scroll_preview(&mut self, delta: i32) {
        if self.preview_visible {
            let new_scroll = self.preview_scroll as i32 + delta;
            self.preview_scroll = new_scroll.max(0) as usize;
        }
    }

    fn toggle_preview(&mut self) {
        if self.options.preview.is_some() {
            self.preview_visible = !self.preview_visible;
        }
    }

    fn toggle_selection(&mut self) {
        if let Some(selected) = self.selection.selected() {
            if let Some(result) = self.filtered_items.get(selected) {
                let item = Arc::clone(&result.item);
                if let Some(pos) = self.multi_selection.iter().position(|i| i.index() == item.index()) {
                    self.multi_selection.remove(pos);
                } else if self.multi_selection.len() < self.options.multi.unwrap_or(usize::MAX) {
                    self.multi_selection.push(item);
                }
                if selected < self.filtered_items.len().saturating_sub(1) {
                    self.selection.select(Some(selected + 1));
                }
            }
        }
    }

    fn accept_selection(&mut self) -> Result<()> {
        if self.multi_selection.is_empty() {
            if let Some(selected) = self.selection.selected() {
                if let Some(result) = self.filtered_items.get(selected) {
                    self.multi_selection.push(Arc::clone(&result.item));
                }
            } else if self.options.exit_0 && self.filtered_items.is_empty() {
                self.running.store(false, Ordering::Relaxed);
                return Ok(());
            } else if let Some(first) = self.filtered_items.first() {
                self.multi_selection.push(Arc::clone(&first.item));
            }
        }
        self.running.store(false, Ordering::Relaxed);
        Ok(())
    }

    fn output_selections(&self) -> Result<()> {
        let delim = if self.options.print0 { '\0' } else { '\n' };
        if self.multi_selection.is_empty() {
            return Ok(());
        }
        for (i, item) in self.multi_selection.iter().enumerate() {
            print!("{}", item.as_string(self.options.ansi));
            if i < self.multi_selection.len() - 1 || !self.options.print0 {
                print!("{}", delim);
            }
        }
        let _ = std::io::stdout().flush();
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();

        let main_area = match self.height_mode {
            HeightMode::Fullscreen => area,
            HeightMode::Fixed(height) => {
                let y = area.height.saturating_sub(height) / 2;
                Rect::new(0, y, area.width, height.min(area.height))
            }
            HeightMode::Percentage(pct) => {
                let height = (area.height as u16 * pct / 100).max(10);
                let y = area.height.saturating_sub(height) / 2;
                Rect::new(0, y, area.width, height.min(area.height))
            }
            HeightMode::Auto(max_pct) => {
                let content_height = self.filtered_items.len().min(20) as u16 + 4;
                let max_height = (area.height as u16 * max_pct / 100).max(10);
                let height = content_height.min(max_height).min(area.height);
                let y = area.height.saturating_sub(height) / 2;
                Rect::new(0, y, area.width, height)
            }
        };

        if self.preview_visible && self.options.preview.is_some() {
            let chunks = RatatuiLayout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(main_area);

            self.draw_main_panel(frame, chunks[0]);
            self.draw_preview_panel(frame, chunks[1]);
        } else {
            self.draw_main_panel(frame, main_area);
        }
    }

    fn draw_main_panel(&mut self, frame: &mut Frame, area: Rect) {
        let border_type = self.get_border_type();
        let has_border = border_type != Borders::NONE;

        let inner_area = if has_border {
            let block = Block::default()
                .borders(border_type)
                .border_type(self.get_border_style())
                .border_style(Style::default().fg(Color::Gray));
            frame.render_widget(block, area);
            area.inner(Margin::new(1, 1))
        } else {
            area
        };

        let chunks = self.layout_chunks(inner_area);

        if let Some(header) = &self.options.header {
            self.draw_header(frame, chunks.header, header);
        }

        self.draw_input(frame, chunks.input);
        self.draw_list(frame, chunks.list);

        if let Some(footer) = &self.options.footer {
            self.draw_footer(frame, chunks.footer, footer);
        }

        if self.info_style != InfoStyle::Hidden {
            self.draw_info(frame, chunks.info);
        }
    }

    fn get_border_type(&self) -> Borders {
        match self.options.border {
            BorderStyle::None => Borders::NONE,
            BorderStyle::Rounded | BorderStyle::Sharp | BorderStyle::Bold |
            BorderStyle::Block | BorderStyle::ThinBlock | BorderStyle::Double => Borders::ALL,
            BorderStyle::Horizontal => Borders::TOP | Borders::BOTTOM,
            BorderStyle::Vertical => Borders::LEFT | Borders::RIGHT,
            BorderStyle::Top => Borders::TOP,
            BorderStyle::Bottom => Borders::BOTTOM,
            BorderStyle::Left => Borders::LEFT,
            BorderStyle::Right => Borders::RIGHT,
        }
    }

    fn get_border_style(&self) -> BorderType {
        match self.options.border {
            BorderStyle::Rounded => BorderType::Rounded,
            BorderStyle::Sharp | BorderStyle::Block | BorderStyle::ThinBlock => BorderType::Plain,
            BorderStyle::Double => BorderType::Double,
            BorderStyle::Bold => BorderType::Thick,
            _ => BorderType::Plain,
        }
    }

    fn draw_preview_panel(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title("Preview")
            .borders(Borders::ALL)
            .border_type(self.get_border_style())
            .border_style(Style::default().fg(Color::Gray));

        let preview_text = if self.preview_output.is_empty() {
            "No preview available"
        } else {
            &self.preview_output
        };

        let lines: Vec<&str> = preview_text.lines().skip(self.preview_scroll).collect();
        let display_text = lines.join("\n");

        let paragraph = Paragraph::new(display_text)
            .block(block)
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, area);
    }

    fn layout_chunks(&self, area: Rect) -> LayoutChunks {
        let has_header = self.options.header.is_some();
        let has_footer = self.options.footer.is_some();

        let constraints = match self.options.layout {
            Layout::Reverse => {
                vec![
                    if has_footer { Constraint::Length(1) } else { Constraint::Length(0) },
                    Constraint::Length(1),
                    Constraint::Min(1),
                    Constraint::Length(1),
                    if has_header { Constraint::Length(1) } else { Constraint::Length(0) },
                ]
            }
            Layout::ReverseList => {
                vec![
                    if has_header { Constraint::Length(1) } else { Constraint::Length(0) },
                    Constraint::Length(1),
                    Constraint::Min(1),
                    Constraint::Length(1),
                    if has_footer { Constraint::Length(1) } else { Constraint::Length(0) },
                ]
            }
            _ => {
                vec![
                    if has_header { Constraint::Length(1) } else { Constraint::Length(0) },
                    Constraint::Length(1),
                    Constraint::Min(1),
                    Constraint::Length(1),
                    if has_footer { Constraint::Length(1) } else { Constraint::Length(0) },
                ]
            }
        };

        let main_chunks = RatatuiLayout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);

        LayoutChunks {
            header: main_chunks[0],
            input: main_chunks[1],
            list: main_chunks[2],
            info: main_chunks[3],
            footer: main_chunks[4],
        }
    }

    fn draw_header(&self, frame: &mut Frame, area: Rect, header: &str) {
        let paragraph = Paragraph::new(header.to_string())
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Left);
        frame.render_widget(paragraph, area);
    }

    fn draw_input(&self, frame: &mut Frame, area: Rect) {
        let border_style = match self.options.border {
            BorderStyle::None => Borders::NONE,
            _ => Borders::BOTTOM,
        };

        let block = Block::default()
            .borders(border_style)
            .border_style(Style::default().fg(Color::Gray));

        let prompt = &self.options.prompt;
        let input_text = format!("{}{}", prompt, self.query);

        let paragraph = Paragraph::new(input_text)
            .block(block)
            .style(Style::default().fg(Color::White));

        frame.render_widget(paragraph, area);
    }

    fn draw_list(&mut self, frame: &mut Frame, area: Rect) {
        let visible_count = area.height as usize;
        let selected = self.selection.selected().unwrap_or(0);

        let start = if selected >= self.scroll_offset + visible_count {
            selected.saturating_sub(visible_count / 2)
        } else if selected < self.scroll_offset {
            selected.saturating_sub(visible_count / 2)
        } else {
            self.scroll_offset
        };

        let end = (start + visible_count).min(self.filtered_items.len());
        self.scroll_offset = start;

        let items: Vec<ListItem> = self.filtered_items[start..end]
            .iter()
            .enumerate()
            .map(|(idx, result)| {
                let actual_idx = start + idx;
                let item = &result.item;
                let text = item.as_string(self.options.ansi);
                let is_selected = self.multi_selection.iter().any(|i| i.index() == item.index());
                let marker = if is_selected { &self.options.marker } else { "  " };
                let display_text = format!("{}{}", marker, text);

                let style = if Some(actual_idx) == self.selection.selected() {
                    Style::default()
                        .bg(Color::Blue)
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                ListItem::new(display_text).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::NONE))
            .highlight_symbol(&self.options.pointer)
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));

        frame.render_stateful_widget(list, area, &mut self.selection);
    }

    fn draw_footer(&self, frame: &mut Frame, area: Rect, footer: &str) {
        let paragraph = Paragraph::new(footer.to_string())
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Left);
        frame.render_widget(paragraph, area);
    }

    fn draw_info(&self, frame: &mut Frame, area: Rect) {
        let info = format!(
            "{}/{} {} {}/{}",
            self.filtered_items.len(),
            self.items.len(),
            self.query,
            self.selection.selected().map_or(0, |i| i + 1),
            self.filtered_items.len()
        );

        let alignment = match self.info_style {
            InfoStyle::Right | InfoStyle::InlineRight => Alignment::Right,
            _ => Alignment::Left,
        };

        let paragraph = Paragraph::new(info)
            .style(Style::default().fg(Color::Gray))
            .alignment(alignment);

        frame.render_widget(paragraph, area);
    }
}

struct LayoutChunks {
    header: Rect,
    input: Rect,
    list: Rect,
    info: Rect,
    footer: Rect,
}
