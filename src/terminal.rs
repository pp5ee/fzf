use crate::item::Item;
use crate::options::{BorderStyle, Options};
use crate::reader::Reader;
use crate::result::Result as FzfResult;
use crate::util::chars::ChunkList;
use crate::util::eventbox::EventBox;
use crate::util::Executor;
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CrosstermEvent, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout as RatatuiLayout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal as RatatuiTerminal,
};
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

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
            reader: Some(reader),
            chunk_list,
            event_box,
            running: Arc::new(AtomicBool::new(true)),
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
                            self.handle_key_event(key.code, key.modifiers)?;
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
            // Show all items when no query
            self.filtered_items = self.items.iter()
                .map(|item| FzfResult::new(
                    Arc::clone(item),
                    crate::result::Rank::new(0, item.index() as u32, item.trim_length()),
                    crate::algo::MatchResult::new(),
                ))
                .collect();

            // Sort by index
            if !self.options.tac {
                self.filtered_items.sort_by_key(|r| r.item.index());
            } else {
                self.filtered_items.sort_by_key(|r| std::cmp::Reverse(r.item.index()));
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

        self.last_search = Instant::now();
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
            KeyCode::Home => self.cursor_pos = 0,
            KeyCode::End => self.cursor_pos = self.query.len(),
            KeyCode::Enter => self.accept_selection()?,
            KeyCode::Tab => {
                if self.options.multi.is_some() {
                    self.toggle_selection();
                }
            }
            KeyCode::Esc => self.running.store(false, Ordering::Relaxed),
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
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let chunks = self.layout_chunks(frame.area());

        if let Some(header) = &self.options.header {
            self.draw_header(frame, chunks.header, header);
        }

        self.draw_input(frame, chunks.input);
        self.draw_list(frame, chunks.list);

        if let Some(footer) = &self.options.footer {
            self.draw_footer(frame, chunks.footer, footer);
        }

        self.draw_info(frame, chunks.info);
    }

    fn layout_chunks(&self, area: Rect) -> LayoutChunks {
        let has_header = self.options.header.is_some();
        let has_footer = self.options.footer.is_some();

        let main_constraints = [
            if has_header { Constraint::Length(1) } else { Constraint::Length(0) },
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
            if has_footer { Constraint::Length(1) } else { Constraint::Length(0) },
        ];

        let main_chunks = RatatuiLayout::default()
            .direction(Direction::Vertical)
            .constraints(main_constraints)
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
}
