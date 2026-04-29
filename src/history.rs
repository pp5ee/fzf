use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, Write};
use std::path::Path;

pub struct History {
    file_path: Option<String>,
    commands: VecDeque<String>,
    max_size: usize,
    index: usize,
}

impl History {
    pub fn new(file_path: Option<String>, max_size: usize) -> Self {
        let mut history = Self {
            file_path,
            commands: VecDeque::new(),
            max_size,
            index: 0,
        };

        if let Err(e) = history.load() {
            eprintln!("Failed to load history: {}", e);
        }

        history
    }

    pub fn load(&mut self) -> io::Result<()> {
        let Some(ref path) = self.file_path else {
            return Ok(());
        };

        if !Path::new(path).exists() {
            return Ok(());
        }

        let file = File::open(path)?;
        let reader = io::BufReader::new(file);

        self.commands.clear();
        for line in reader.lines() {
            let line = line?;
            if !line.is_empty() {
                self.commands.push_back(line);
                if self.commands.len() > self.max_size {
                    self.commands.pop_front();
                }
            }
        }

        self.index = self.commands.len();
        Ok(())
    }

    pub fn save(&self) -> io::Result<()> {
        let Some(ref path) = self.file_path else {
            return Ok(());
        };

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;

        for cmd in &self.commands {
            writeln!(file, "{}", cmd)?;
        }

        Ok(())
    }

    pub fn add(&mut self, command: String) {
        if command.is_empty() {
            return;
        }

        // Avoid duplicates at the end
        if self.commands.back() != Some(&command) {
            self.commands.push_back(command);
            if self.commands.len() > self.max_size {
                self.commands.pop_front();
            }
        }

        self.index = self.commands.len();
    }

    pub fn prev(&mut self) -> Option<&String> {
        if self.index > 0 {
            self.index -= 1;
            self.commands.get(self.index)
        } else {
            None
        }
    }

    pub fn next(&mut self) -> Option<&String> {
        if self.index < self.commands.len() {
            self.index += 1;
            self.commands.get(self.index)
        } else {
            None
        }
    }

    pub fn get(&self, index: usize) -> Option<&String> {
        self.commands.get(index)
    }

    pub fn len(&self) -> usize {
        self.commands.len()
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    pub fn reset_index(&mut self) {
        self.index = self.commands.len();
    }

    pub fn commands(&self) -> &VecDeque<String> {
        &self.commands
    }
}

impl Default for History {
    fn default() -> Self {
        Self::new(None, 1000)
    }
}
