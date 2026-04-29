pub mod chars;
pub mod eventbox;
pub mod slab;

use std::sync::atomic::{AtomicBool, Ordering};

pub struct AtomicBoolWrapper {
    inner: AtomicBool,
}

impl AtomicBoolWrapper {
    pub fn new(value: bool) -> Self {
        Self {
            inner: AtomicBool::new(value),
        }
    }

    pub fn load(&self, ordering: Ordering) -> bool {
        self.inner.load(ordering)
    }

    pub fn store(&self, value: bool, ordering: Ordering) {
        self.inner.store(value, ordering);
    }

    pub fn swap(&self, value: bool, ordering: Ordering) -> bool {
        self.inner.swap(value, ordering)
    }

    pub fn compare_and_swap(&self, current: bool, new: bool, ordering: Ordering) -> bool {
        self.inner
            .compare_exchange(current, new, ordering, Ordering::Relaxed)
            .unwrap_or_else(|x| x)
    }
}

impl Default for AtomicBoolWrapper {
    fn default() -> Self {
        Self::new(false)
    }
}

impl Clone for AtomicBoolWrapper {
    fn clone(&self) -> Self {
        Self::new(self.load(Ordering::Relaxed))
    }
}

pub struct Executor;

impl Executor {
    pub fn new() -> Self {
        Self
    }

    pub fn execute(&self, command: &str) -> anyhow::Result<Vec<u8>> {
        use std::process::Command;

        let output = if cfg!(target_os = "windows") {
            Command::new("cmd").args(["/C", command]).output()?
        } else {
            Command::new("sh").arg("-c").arg(command).output()?
        };

        if output.status.success() {
            Ok(output.stdout)
        } else {
            Err(anyhow::anyhow!(
                "Command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}
