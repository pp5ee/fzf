pub mod light;

use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    Key(Key),
    Mouse(MouseEvent),
    Resize(u16, u16),
    FocusGained,
    FocusLost,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Key {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MouseEvent {
    pub kind: MouseEventKind,
    pub column: u16,
    pub row: u16,
    pub modifiers: KeyModifiers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseEventKind {
    Down(MouseButton),
    Up(MouseButton),
    Drag(MouseButton),
    Moved,
    ScrollDown,
    ScrollUp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

impl From<CrosstermEvent> for Event {
    fn from(event: CrosstermEvent) -> Self {
        match event {
            CrosstermEvent::Key(key) => Event::Key(Key {
                code: key.code,
                modifiers: key.modifiers,
            }),
            CrosstermEvent::Mouse(mouse) => {
                let kind = match mouse.kind {
                    crossterm::event::MouseEventKind::Down(btn) => {
                        MouseEventKind::Down(match btn {
                            crossterm::event::MouseButton::Left => MouseButton::Left,
                            crossterm::event::MouseButton::Right => MouseButton::Right,
                            crossterm::event::MouseButton::Middle => MouseButton::Middle,
                        })
                    }
                    crossterm::event::MouseEventKind::Up(btn) => {
                        MouseEventKind::Up(match btn {
                            crossterm::event::MouseButton::Left => MouseButton::Left,
                            crossterm::event::MouseButton::Right => MouseButton::Right,
                            crossterm::event::MouseButton::Middle => MouseButton::Middle,
                        })
                    }
                    crossterm::event::MouseEventKind::Drag(btn) => {
                        MouseEventKind::Drag(match btn {
                            crossterm::event::MouseButton::Left => MouseButton::Left,
                            crossterm::event::MouseButton::Right => MouseButton::Right,
                            crossterm::event::MouseButton::Middle => MouseButton::Middle,
                        })
                    }
                    crossterm::event::MouseEventKind::Moved => MouseEventKind::Moved,
                    crossterm::event::MouseEventKind::ScrollDown => MouseEventKind::ScrollDown,
                    crossterm::event::MouseEventKind::ScrollUp => MouseEventKind::ScrollUp,
                    _ => MouseEventKind::Moved,
                };
                Event::Mouse(MouseEvent {
                    kind,
                    column: mouse.column,
                    row: mouse.row,
                    modifiers: mouse.modifiers,
                })
            }
            CrosstermEvent::Resize(w, h) => Event::Resize(w, h),
            CrosstermEvent::FocusGained => Event::FocusGained,
            CrosstermEvent::FocusLost => Event::FocusLost,
            _ => Event::None,
        }
    }
}
