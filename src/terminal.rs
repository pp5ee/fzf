use crate::item::Item;
use crate::matcher::Matcher;
use crate::merger::Merger;
use crate::options::{BorderStyle, InfoStyle, Layout, Options};
use crate::pattern::{Pattern, PatternOptions};
use crate::reader::{Reader, EVT_READ_FIN, EVT_READ_NEW};
use crate::result::Result as FzfResult;
use crate::util::chars::ChunkList;
use crate::util::eventbox::{EventBox, EventValue};
use crate::util::Executor;
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CrosstermEvent, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout as RatatuiLayout, Margin, Rect},
    style::{Color, Modifier, Style},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal as RatatuiTerminal,
};
use std::io;
use std::sync::atomic::{AtomicBool, Ordering, AtomicUsize};
use std::sync::Arc;
use std::time::{Duration, Instant};

static ITEM_INDEX: AtomicUsize = AtomicUsize::new(0);

pub struct Terminal {
    options: Options,
    query: String,
    cursor_pos: usize,
    items: Vec<Arc<Item>>,
    filtered_items: Vec<FzfResult>,
    selection: ListState,
    scroll_offset: usize,
    reader: Option<Reader>,
    matcher: Option<Matcher>,
    chunk_list: Arc<std::sync::Mutex<ChunkList>>,
    event_box: EventBox,
    running: Arc<AtomicBool>,
    preview_visible: bool,
    preview_command: Option<String>,
    preview_output: String,
    multi_selection: Vec<Arc<Item>>,
    last_search: Instant,
    search_debounce: Duration,
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

        Self {
            options,
            query: String::new(),
            cursor_pos: 0,
            items: Vec::new(),
            filtered_items: Vec::new(),
            selection: ListState::default(),
            scroll_offset: 0,
            reader: Some(reader),
            matcher: None,
            chunk_list,
            event_box,
            running: Arc::new(AtomicBool::new(true)),
            preview_visible: false,
            preview_command: None,
            preview_output: String::new(),
            multi_selection: Vec::new(),
            last_search: Instant::now(),
            search_debounce: Duration::from_millis(50),
        }
    }

    pub fn run(&mut self) -> Result<i32> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = RatatuiTerminal::new(backend)?;

        // Start reader thread
        let reader = self.reader.take().unwrap();
        let chunk_list_clone = Arc::clone(&self.chunk_list);

        std::thread::spawn(move || {
            let _ = reader.read_stdin();
        });

        // Load initial items
        self.load_items();

        // Main event loop
        let result = self.run_event_loop(&mut terminal);

        // Cleanup
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;

        // Output selected items
        if result.is_ok() {
            self.output_selections()?;
        }

        result
    }

    fn run_event_loop<B: Backend>(&mut self, terminal: &mut RatatuiTerminal<B>) -> Result<i32> {
        let mut last_tick = Instant::now();
        let tick_rate = Duration::from_millis(100);

        while self.running.load(Ordering::Relaxed) {
            // Draw UI
            terminal.draw(|f| self.draw(f))?;

            // Handle events
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if crossterm::event::poll(timeout)? {
                if let Ok(event) = event::read() {
                    if let CrosstermEvent::Key(key) = event {
                        if key.kind == KeyEventKind::Press {
                            self.handle_key_event(key.code)?;
                        }
                    }
                }
            }

            // Periodic updates
            if last_tick.elapsed() >= tick_rate {
                self.load_items();
                self.perform_search();
                last_tick = Instant::now();
            }

            // Check for new input
            if let Some((evt, _)) = self.event_box.try_recv() {
                if evt == EVT_READ_FIN || evt == EVT_READ_NEW {
                    self.load_items();
                }
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
        let mut new_items = 0;

        for chunk in list.chunks() {
            for chars in &chunk.items {
                if chars.index() >= start_index as i32 {
                    self.items.push(Arc::new(Item::new(chars.clone())));
                    new_items += 1;
                }
            }
        }

        if new_items > 0 && self.query.is_empty() {
            // Show all items when no query
            self.filtered_items = self.items[start_index..]
                .iter()
                .map(|item| FzfResult::new(
                    Arc::clone(item),
                    crate::result::Rank::new(0, item.index() as u32, item.trim_length()),
                    crate::algo::MatchResult::new(),
                ))
                .collect();

            // Sort by index if needed
            if !self.options.tac {
                self.filtered_items.sort_by_key(|r| r.item.index());
            } else {
                self.filtered_items.sort_by_key(|r| -(r.item.index()));
            }
        }
    }

    fn perform_search(&mut self) {
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

        let pattern = Pattern::new(&self.query, &pattern_opts);

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

        // Sort by score
        if !self.options.no_sort {
            results.sort_by(|a, b| {
                b.rank.score.cmp(&a.rank.score)
                    .then_with(|| a.rank.length.cmp(&b.rank.length))
                    .then_with(|| b.rank.index.cmp(&a.rank.index))
            });
        }

        self.filtered_items = results;
        self.last_search = Instant::now();

        // Reset selection if out of bounds
        if let Some(selected) = self.selection.selected() {
            if selected >= self.filtered_items.len() {
                self.selection.select(if self.filtered_items.is_empty() {
                    None
                } else {
                    Some(self.filtered_items.len() - 1)
                });
            }
        }
    }

    fn handle_key_event(&mut self, code: KeyCode) -> Result<()> {
        match code {
            KeyCode::Char(c) => {
                self.query.insert(self.cursor_pos, c);
                self.cursor_pos += 1;
                self.last_search = Instant::now() - self.search_debounce;
            }
            KeyCode::Backspace => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                    self.query.remove(self.cursor_pos);
                    self.last_search = Instant::now() - self.search_debounce;
                }
            }
            KeyCode::Left => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                }
            }
            KeyCode::Right => {
                if self.cursor_pos < self.query.len() {
                    self.cursor_pos += 1;
                }
            }
            KeyCode::Up => {
                if let Some(selected) = self.selection.selected() {
                    if selected > 0 {
                        self.selection.select(Some(selected - 1));
                    }
                } else if !self.filtered_items.is_empty() {
                    self.selection.select(Some(self.filtered_items.len() - 1));
                }
            }
            KeyCode::Down => {
                if let Some(selected) = self.selection.selected() {
                    if selected < self.filtered_items.len().saturating_sub(1) {
                        self.selection.select(Some(selected + 1));
                    }
                } else if !self.filtered_items.is_empty() {
                    self.selection.select(Some(0));
                }
            }
            KeyCode::Home => {
                self.cursor_pos = 0;
            }
            KeyCode::End => {
                self.cursor_pos = self.query.len();
            }
            KeyCode::Enter => {
                self.accept_selection()?;
            }
            KeyCode::Tab => {
                if self.options.multi.is_some() {
                    self.toggle_selection();
                }
            }
            KeyCode::Esc => {
                self.running.store(false, Ordering::Relaxed);
            }
            KeyCode::Ctrl('c') => {
                self.running.store(false, Ordering::Relaxed);
            }
            KeyCode::Ctrl('u') => {
                self.query.clear();
                self.cursor_pos = 0;
            }
            KeyCode::Ctrl('w') => {
                self.delete_word();
            }
            KeyCode::Ctrl('a') => {
                self.cursor_pos = 0;
            }
            KeyCode::Ctrl('e') => {
                self.cursor_pos = self.query.len();
            }
            KeyCode::F(1..=12) => {}
            _ => {}
        }
        Ok(())
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

    fn toggle_selection(&mut self) {
        if let Some(selected) = self.selection.selected() {
            if let Some(result) = self.filtered_items.get(selected) {
                let item = Arc::clone(&result.item);
                if let Some(pos) = self.multi_selection.iter().position(|i| i.index() == item.index()) {
                    self.multi_selection.remove(pos);
                } else if self.multi_selection.len() < self.options.multi.unwrap_or(usize::MAX) {
                    self.multi_selection.push(item);
                }

                // Move down after toggle
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
            // No selection made
            return Ok(());
        }

        for (i, item) in self.multi_selection.iter().enumerate() {
            print!("{}", item.as_string(self.options.ansi));
            if i < self.multi_selection.len() - 1 || !self.options.print0 {
                print!("{}", delim);
            }
        }

        Ok(())
    }

    fn draw<B: Backend>(&mut self, frame: &mut Frame<B>) {
        let chunks = self.layout_chunks(frame.size());

        // Draw header
        if let Some(header) = &self.options.header {
            self.draw_header(frame, chunks.header, header);
        }

        // Draw input
        self.draw_input(frame, chunks.input);

        // Draw list
        self.draw_list(frame, chunks.list);

        // Draw preview if visible
        if self.preview_visible && self.preview_command.is_some() {
            self.draw_preview(frame, chunks.preview);
        }

        // Draw footer
        if let Some(footer) = &self.options.footer {
            self.draw_footer(frame, chunks.footer, footer);
        }

        // Draw info line
        self.draw_info(frame, chunks.info);
    }

    fn layout_chunks(&self, area: Rect) -> LayoutChunks {
        let has_header = self.options.header.is_some();
        let has_footer = self.options.footer.is_some();
        let has_preview = self.preview_visible && self.preview_command.is_some();

        let main_constraints = [
            if has_header { Constraint::Length(1) } else { Constraint::Length(0) },
            Constraint::Length(1), // Input
            Constraint::Min(1),    // List
            Constraint::Length(1), // Info
            if has_footer { Constraint::Length(1) } else { Constraint::Length(0) },
        ];

        let main_chunks = RatatuiLayout::default()
            .direction(Direction::Vertical)
            .constraints(main_constraints)
            .split(area);

        let list_area = if has_preview {
            let list_preview = RatatuiLayout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(main_chunks[2]);
            list_preview[0]
        } else {
            main_chunks[2]
        };

        LayoutChunks {
            header: main_chunks[0],
            input: main_chunks[1],
            list: list_area,
            info: main_chunks[3],
            footer: main_chunks[4],
            preview: if has_preview {
                let list_preview = RatatuiLayout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(main_chunks[2]);
                list_preview[1]
            } else {
                Rect::default()
            },
        }
    }

    fn draw_header<B: Backend>(&self, frame: &mut Frame<B>, area: Rect, header: &str) {
        let paragraph = Paragraph::new(header.clone())
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Left);
        frame.render_widget(paragraph, area);
    }

    fn draw_input<B: Backend>(&self, frame: &mut Frame<B>, area: Rect) {
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

    fn draw_list<B: Backend>(&mut self, frame: &mut Frame<B>, area: Rect) {
        let items: Vec<ListItem> = self.filtered_items
            .iter()
            .enumerate()
            .map(|(idx, result)| {
                let item = &result.item;
                let text = item.as_string(self.options.ansi);
                let is_selected = self.multi_selection.iter().any(|i| i.index() == item.index());
                let marker = if is_selected { &self.options.marker } else { "  " };

                let display_text = format!("{}{}", marker, text);

                let style = if Some(idx) == self.selection.selected() {
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

    fn draw_preview<B: Backend>(&self, frame: &mut Frame<B>, area: Rect) {
        let block = Block::default()
            .title("Preview")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Gray));

        let paragraph = Paragraph::new(self.preview_output.clone())
            .block(block)
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, area);
    }

    fn draw_footer<B: Backend>(&self, frame: &mut Frame<B>, area: Rect, footer: &str) {
        let paragraph = Paragraph::new(footer.to_string())
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Left);
        frame.render_widget(paragraph, area);
    }

    fn draw_info<B: Backend>(&self, frame: &mut Frame<B>, area: Rect) {
        let info = format!(
            "{}/{} {} {}/{}",
            self.filtered_items.len(),
            self.items.len(),
            self.query,
            self.selection.selected().map_or(0, |i| i + 1),
            self.filtered_items.len()
        );

        let paragraph = Paragraph::new(info)
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Right);

        frame.render_widget(paragraph, area);
    }
}

struct LayoutChunks {
    header: Rect,
    input: Rect,
    list: Rect,
    info: Rect,
    footer: Rect,
    preview: Rect,
}

use std::sync::atomic::Ordering as AtomicOrdering;
