use std::sync::Mutex;

pub struct Slab<T> {
    items: Mutex<Vec<T>>,
    capacity: usize,
}

impl<T: Default + Clone> Slab<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            items: Mutex::new(Vec::with_capacity(capacity)),
            capacity,
        }
    }

    pub fn alloc(&self, count: usize) -> Vec<T> {
        let mut items = self.items.lock().unwrap();
        let len = items.len();

        if len >= count {
            items.split_off(len - count)
        } else {
            vec![T::default(); count]
        }
    }

    pub fn free(&self, mut data: Vec<T>) {
        let mut items = self.items.lock().unwrap();

        if items.len() + data.len() <= self.capacity {
            data.clear();
            items.extend(data);
        }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SlabEntry {
    pub value: i32,
}

pub type IntSlab = Slab<SlabEntry>;
