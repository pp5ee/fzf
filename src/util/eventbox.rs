use crossbeam::channel::{bounded, unbounded, Receiver, Sender};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

pub type EventType = u32;

#[derive(Debug, Clone)]
pub enum EventValue {
    None,
    Int(i32),
    Bool(bool),
    String(String),
}

impl Default for EventValue {
    fn default() -> Self {
        EventValue::None
    }
}

pub struct EventBox {
    sender: Sender<(EventType, EventValue)>,
    receiver: Receiver<(EventType, EventValue)>,
    events: Arc<Mutex<HashMap<EventType, EventValue>>>,
}

impl EventBox {
    pub fn new() -> Self {
        let (sender, receiver) = unbounded();
        Self {
            sender,
            receiver,
            events: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let (sender, receiver) = bounded(capacity);
        Self {
            sender,
            receiver,
            events: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn set(&self, event_type: EventType, value: EventValue) {
        let _ = self.sender.send((event_type, value));
    }

    pub fn set_sync(&self, event_type: EventType, value: EventValue) {
        self.events.lock().insert(event_type, value);
    }

    pub fn get(&self, event_type: EventType) -> Option<EventValue> {
        self.events.lock().get(&event_type).cloned()
    }

    pub fn wait<F>(&self, mut f: F)
    where
        F: FnMut(&mut HashMap<EventType, EventValue>),
    {
        let mut events = self.events.lock();

        while let Ok((event_type, value)) = self.receiver.try_recv() {
            events.insert(event_type, value);
        }

        f(&mut events);
    }

    pub fn try_recv(&self) -> Option<(EventType, EventValue)> {
        self.receiver.try_recv().ok()
    }

    pub fn recv(&self) -> Option<(EventType, EventValue)> {
        self.receiver.recv().ok()
    }

    pub fn recv_timeout(&self, timeout: std::time::Duration) -> Option<(EventType, EventValue)> {
        self.receiver.recv_timeout(timeout).ok()
    }
}

impl Default for EventBox {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for EventBox {
    fn clone(&self) -> Self {
        let (sender, receiver) = unbounded();
        Self {
            sender,
            receiver,
            events: Arc::clone(&self.events),
        }
    }
}
