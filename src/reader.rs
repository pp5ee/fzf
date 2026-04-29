use crate::util::chars::{Chars, Chunk, ChunkList};
use crate::util::eventbox::{EventBox, EventType, EventValue};
use crate::util::Executor;
use std::io::{self, BufRead, Read};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;

pub const EVT_READ_FIN: EventType = 10;
pub const EVT_READ_NEW: EventType = 11;
pub const EVT_READY: i32 = 0;

pub struct Reader {
    chunk_list: Arc<std::sync::Mutex<ChunkList>>,
    executor: Executor,
    event_box: EventBox,
    delim_null: bool,
    event: AtomicI32,
    command: Option<String>,
}

impl Reader {
    pub fn new(
        event_box: EventBox,
        executor: Executor,
        delim_null: bool,
    ) -> Self {
        let chunk_list = Arc::new(std::sync::Mutex::new(ChunkList::new(1000)));

        Self {
            chunk_list,
            executor,
            event_box,
            delim_null,
            event: AtomicI32::new(EVT_READY),
            command: None,
        }
    }

    pub fn set_command(&mut self, command: String) {
        self.command = Some(command);
    }

    pub fn read_stdin(&self) -> io::Result<()> {
        let stdin = io::stdin();
        let mut stdin = stdin.lock();

        let delimiter = if self.delim_null { b'\0' } else { b'\n' };
        let mut buffer = Vec::new();
        let mut index = 0i32;

        loop {
            buffer.clear();
            let bytes_read = if self.delim_null {
                read_until(&mut stdin, delimiter, &mut buffer)?
            } else {
                stdin.read_until(delimiter, &mut buffer)?
            };

            if bytes_read == 0 {
                break;
            }

            // Remove trailing delimiter
            if !buffer.is_empty() && buffer.last() == Some(&delimiter) {
                buffer.pop();
            }

            if !buffer.is_empty() {
                let chars = Chars::from_bytes(buffer.clone())
                    .with_index(index);

                {
                    let mut list = self.chunk_list.lock().unwrap();
                    list.push(chars);
                }

                index += 1;

                if index % 100 == 0 {
                    self.event_box.set_sync(EVT_READ_NEW, EventValue::None);
                }
            }
        }

        self.event_box.set(EVT_READ_FIN, EventValue::None);
        Ok(())
    }

    pub fn read_command(&self) -> io::Result<()> {
        if let Some(ref cmd) = self.command {
            match self.executor.execute(cmd) {
                Ok(output) => {
                    let mut list = self.chunk_list.lock().unwrap();
                    let delimiter = if self.delim_null { b'\0' } else { b'\n' };
                    let mut index = 0i32;

                    for line in output.split(|&b| b == delimiter) {
                        if !line.is_empty() {
                            let chars = Chars::from_bytes(line.to_vec())
                                .with_index(index);
                            list.push(chars);
                            index += 1;
                        }
                    }
                }
                Err(_) => {}
            }
        }

        self.event_box.set(EVT_READ_FIN, EventValue::None);
        Ok(())
    }

    pub fn chunk_list(&self) -> Arc<std::sync::Mutex<ChunkList>> {
        Arc::clone(&self.chunk_list)
    }

    pub fn is_finished(&self) -> bool {
        self.event.load(Ordering::Relaxed) == EVT_READ_FIN as i32
    }
}

fn read_until<R: Read>(reader: &mut R, byte: u8, buf: &mut Vec<u8>) -> io::Result<usize> {
    let mut count = 0;
    let mut byte_buf = [0u8; 1];

    loop {
        match reader.read(&mut byte_buf) {
            Ok(0) => return Ok(count),
            Ok(1) => {
                buf.push(byte_buf[0]);
                count += 1;
                if byte_buf[0] == byte {
                    return Ok(count);
                }
            }
            Ok(n) => unreachable!("unexpected read size: {}", n),
            Err(e) => return Err(e),
        }
    }
}
