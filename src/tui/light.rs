use super::{Event, Key};
use crossterm::{
    event::{self, Event as CrosstermEvent},
    terminal::{disable_raw_mode, enable_raw_mode, is_raw_mode_enabled},
};
use std::io;

pub struct LightTerminal {
    raw_mode: bool,
}

impl LightTerminal {
    pub fn new() -> io::Result<Self> {
        Ok(Self { raw_mode: false })
    }

    pub fn init(&mut self) -> io::Result<()> {
        if !is_raw_mode_enabled()? {
            enable_raw_mode()?;
            self.raw_mode = true;
        }
        Ok(())
    }

    pub fn cleanup(&mut self) -> io::Result<()> {
        if self.raw_mode {
            disable_raw_mode()?;
            self.raw_mode = false;
        }
        Ok(())
    }

    pub fn poll_event(&self, timeout_ms: u64) -> io::Result<Option<Event>> {
        if event::poll(std::time::Duration::from_millis(timeout_ms))? {
            let event = event::read()?;
            Ok(Some(event.into()))
        } else {
            Ok(None)
        }
    }

    pub fn read_event(&self) -> io::Result<Event> {
        let event = event::read()?;
        Ok(event.into())
    }
}

impl Drop for LightTerminal {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}

impl Default for LightTerminal {
    fn default() -> Self {
        Self::new().expect("Failed to create LightTerminal")
    }
}
