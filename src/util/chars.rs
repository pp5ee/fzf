use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Clone)]
pub struct Chars {
    bytes: Vec<u8>,
    runes: Option<Vec<char>>,
    index: i32,
    trim_length: Option<u16>,
}

impl Chars {
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        // Check if ASCII
        let is_ascii = bytes.iter().all(|&b| b < 0x80);

        if is_ascii {
            Self {
                bytes,
                runes: None,
                index: 0,
                trim_length: None,
            }
        } else {
            let runes: Vec<char> = String::from_utf8_lossy(&bytes)
                .chars()
                .collect();
            Self {
                bytes,
                runes: Some(runes),
                index: 0,
                trim_length: None,
            }
        }
    }

    pub fn from_string(s: String) -> Self {
        let bytes = s.into_bytes();
        Self::from_bytes(bytes)
    }

    pub fn from_str(s: &str) -> Self {
        Self::from_bytes(s.as_bytes().to_vec())
    }

    pub fn with_index(mut self, index: i32) -> Self {
        self.index = index;
        self
    }

    pub fn is_ascii(&self) -> bool {
        self.runes.is_none()
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.bytes).unwrap_or("")
    }

    pub fn len(&self) -> usize {
        match &self.runes {
            Some(runes) => runes.len(),
            None => self.bytes.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    pub fn index(&self) -> i32 {
        self.index
    }

    pub fn get(&self, idx: usize) -> Option<char> {
        match &self.runes {
            Some(runes) => runes.get(idx).copied(),
            None => self.bytes.get(idx).map(|&b| b as char),
        }
    }

    pub fn to_vec(&self) -> Vec<char> {
        match &self.runes {
            Some(runes) => runes.clone(),
            None => self.bytes.iter().map(|&b| b as char).collect(),
        }
    }

    pub fn trim_length(&self) -> u16 {
        if let Some(len) = self.trim_length {
            return len;
        }

        let len = self.len() as u16;
        len.min(u16::MAX)
    }

    pub fn num_lines(&self, at_most: usize) -> (usize, bool) {
        let mut lines = 1;
        for c in self.to_vec() {
            if c == '\n' {
                lines += 1;
            }
            if lines > at_most {
                return (at_most, true);
            }
        }
        (lines, false)
    }
}

impl Default for Chars {
    fn default() -> Self {
        Self {
            bytes: Vec::new(),
            runes: None,
            index: 0,
            trim_length: None,
        }
    }
}

impl PartialEq for Chars {
    fn eq(&self, other: &Self) -> bool {
        self.bytes == other.bytes
    }
}

impl Eq for Chars {}

#[derive(Debug, Clone)]
pub struct Chunk {
    pub items: Vec<Chars>,
    pub capacity: usize,
}

impl Chunk {
    pub fn new(capacity: usize) -> Self {
        Self {
            items: Vec::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, item: Chars) -> bool {
        if self.items.len() < self.capacity {
            self.items.push(item);
            true
        } else {
            false
        }
    }

    pub fn is_full(&self) -> bool {
        self.items.len() >= self.capacity
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new(1000)
    }
}

pub struct ChunkList {
    chunks: Vec<Chunk>,
    chunk_size: usize,
    total_count: usize,
}

impl ChunkList {
    pub fn new(chunk_size: usize) -> Self {
        Self {
            chunks: Vec::new(),
            chunk_size,
            total_count: 0,
        }
    }

    pub fn push(&mut self, item: Chars) {
        if self.chunks.is_empty() || self.chunks.last().unwrap().is_full() {
            self.chunks.push(Chunk::new(self.chunk_size));
        }

        let chunk = self.chunks.last_mut().unwrap();
        chunk.push(item);
        self.total_count += 1;
    }

    pub fn chunks(&self) -> &[Chunk] {
        &self.chunks
    }

    pub fn total_count(&self) -> usize {
        self.total_count
    }

    pub fn is_empty(&self) -> bool {
        self.total_count == 0
    }

    pub fn clear(&mut self) {
        self.chunks.clear();
        self.total_count = 0;
    }
}

impl Default for ChunkList {
    fn default() -> Self {
        Self::new(1000)
    }
}
