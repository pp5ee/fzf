# fzf Go→Rust Alternative Implementation Strategies

## Overview

This document identifies Go features that cannot be directly translated to Rust and provides alternative implementation strategies with compatibility impact analysis.

---

## 1. Concurrency Model

### 1.1 Goroutines vs Async/Parallel

| Aspect | Go | Rust Challenge | Alternative |
|--------|-----|----------------|-------------|
| **Green threads** | M:N scheduling (goroutines) | No built-in runtime | tokio (async) + rayon (parallel) |
| **Stack size** | 2KB dynamic growth | Fixed stack | tokio tasks + spawning strategy |
| **Preemption** | Cooperative (function calls) | None | Manual yield points or await |

**Go Code**:
```go
// Go: Simple goroutine spawning
func (m *Matcher) Loop() {
    for i := 0; i < m.partitions; i++ {
        go m.worker(i)  // Lightweight, cheap
    }
}
```

**Rust Alternative**:
```rust
// Strategy 1: Rayon for CPU-bound work (recommended)
use rayon::prelude::*;

impl Matcher {
    pub fn scan(&self, chunks: &[Chunk]) -> Vec<Result> {
        chunks
            .par_chunks(chunks.len() / self.partitions)
            .flat_map(|chunk| self.match_chunk(chunk))
            .collect()
    }
}

// Strategy 2: Tokio for I/O-bound work
use tokio::task;

impl Reader {
    pub async fn read_parallel(&self, sources: Vec<Source>) -> Vec<Item> {
        let handles: Vec<_> = sources.into_iter()
            .map(|src| task::spawn(async move {
                self.read_source(src).await
            }))
            .collect();

        let mut results = Vec::new();
        for handle in handles {
            results.extend(handle.await.unwrap());
        }
        results
    }
}
```

**Compatibility Impact**:
| Aspect | Impact | Mitigation |
|--------|--------|------------|
| Task spawn cost | Higher in Rust | Use thread pools (rayon) |
| Memory usage | Stack size fixed | Configure tokio stack size |
| Scheduling | Less automatic | Explicit yield/await points |
| Performance | Similar with tuning | Benchmark and tune |

**Regression Verification**:
```rust
#[test]
fn test_parallel_matching_equivalent() {
    let items = generate_items(100000);
    let pattern = Pattern::new("test");

    // Sequential
    let seq_results = matcher.match_sequential(&items, &pattern);

    // Parallel
    let par_results = matcher.match_parallel(&items, &pattern);

    // Results must be identical (order may differ if unsorted)
    assert_eq!(seq_results.len(), par_results.len());
    assert_eq!(
        seq_results.iter().collect::<HashSet<_>>(),
        par_results.iter().collect::<HashSet<_>>()
    );
}
```

---

### 1.2 Select Statement vs Tokio::select!

| Aspect | Go | Rust Challenge | Alternative |
|--------|-----|----------------|-------------|
| **Select** | Built-in multiplexing | No direct equivalent | tokio::select! macro |
| **Default case** | `default:` | Requires timeout | `tokio::time::timeout` |
| **Random selection** | Fair selection | Deterministic order | Shuffle before select or fair channels |

**Go Code**:
```go
// Go: Event loop with select
for {
    select {
    case event := <-eventCh:
        handleEvent(event)
    case req := <-reqCh:
        handleRequest(req)
    case <-ctx.Done():
        return
    default:
        // Non-blocking
    }
}
```

**Rust Alternative**:
```rust
// Strategy: tokio::select! with bias consideration
loop {
    tokio::select! {
        biased;  // Process in order (not random like Go)

        event = event_rx.recv() => {
            match event {
                Some(e) => self.handle_event(e).await?,
                None => break, // Channel closed
            }
        }
        req = req_rx.recv() => {
            match req {
                Some(r) => self.handle_request(r).await?,
                None => break,
            }
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Received Ctrl+C, shutting down");
            break;
        }
    }
}
```

**Compatibility Impact**:
| Aspect | Impact | Mitigation |
|--------|--------|------------|
| Fairness | Go is random; Rust is ordered | Use `biased;` or shuffle |
| Cancellation | Context in Go | CancellationToken in Rust |
| Blocking | `default:` non-blocking | Use `try_recv()` pattern |

**Regression Verification**:
- Test event ordering behavior
- Verify graceful shutdown
- Test cancellation propagation

---

### 1.3 sync.Cond vs Async Condition Variables

| Aspect | Go | Rust Challenge | Alternative |
|--------|-----|----------------|-------------|
| **sync.Cond** | Broadcast/wait | Not in async Rust | tokio::sync::Notify |
| **Usage pattern** | Lock + Wait | Different API | Notify + watched state |

**Go Code**:
```go
// Go: EventBox with sync.Cond
type EventBox struct {
    cond   *sync.Cond
    events Events
}

func (b *EventBox) Wait(callback func(*Events)) {
    b.cond.L.Lock()
    if len(b.events) == 0 {
        b.cond.Wait()  // Blocks until Broadcast
    }
    callback(&b.events)
    b.cond.L.Unlock()
}

func (b *EventBox) Set(event EventType, value any) {
    b.cond.L.Lock()
    b.events[event] = value
    b.cond.Broadcast()  // Wake all waiters
    b.cond.L.Unlock()
}
```

**Rust Alternative**:
```rust
// Strategy 1: tokio::sync::Notify (for async)
use tokio::sync::Notify;
use std::sync::Mutex;

pub struct AsyncEventBox {
    events: Mutex<Events>,
    notify: Notify,
}

impl AsyncEventBox {
    pub async fn wait<F>(&self, callback: F)
    where
        F: FnOnce(&Events),
    {
        let events = self.events.lock().unwrap();
        if events.is_empty() {
            drop(events); // Release lock before await
            self.notify.notified().await;
        }

        let events = self.events.lock().unwrap();
        callback(&*events);
    }

    pub fn set(&self, event: EventType, value: EventValue) {
        let mut events = self.events.lock().unwrap();
        events.insert(event, value);
        self.notify.notify_waiters(); // Like Broadcast
    }
}

// Strategy 2: tokio::sync::watch (for state changes)
use tokio::sync::watch;

pub struct WatchEventBox {
    tx: watch::Sender<Event>,
}

impl WatchEventBox {
    pub async fn subscribe(&self) -> watch::Receiver<Event> {
        self.tx.subscribe()
    }

    pub fn broadcast(&self, event: Event) {
        let _ = self.tx.send(event);
    }
}
```

**Compatibility Impact**:
| Aspect | Impact | Mitigation |
|--------|--------|------------|
| API shape | Different | Wrapper to match Go interface |
| Performance | Similar | Benchmark both |
| Deadlock risk | Different patterns | Careful lock ordering |

**Regression Verification**:
- Test concurrent event production/consumption
- Verify no lost wakeups
- Test shutdown behavior

---

## 2. Memory Management

### 2.1 Garbage Collection vs Ownership

| Aspect | Go | Rust Challenge | Alternative |
|--------|-----|----------------|-------------|
| **Memory safety** | GC guarantees | Compile-time checks | Ownership + borrow checker |
| **Circular refs** | Handled by GC | Memory leaks possible | Weak references or restructure |
| **Allocation** | Implicit | Explicit | Arena allocators for hot paths |

**Go Code**:
```go
// Go: GC handles cleanup
type Matcher struct {
    cache    *ChunkCache  // Shared, GC tracks
    slab     []*util.Slab // Slice of pointers
}

func (m *Matcher) scan(chunks []*Chunk) {
    for _, chunk := range chunks {
        go m.matchChunk(chunk)  // chunk captured, GC extends lifetime
    }
}
```

**Rust Alternative**:
```rust
// Strategy: Arc for shared ownership
use std::sync::Arc;

pub struct Matcher {
    cache: Arc<ChunkCache>,
    // Slab allocation using arena
    slab_arena: Mutex<Vec<Slab>>,
}

impl Matcher {
    pub fn scan(&self, chunks: &[Arc<Chunk>]) -> Vec<Result> {
        // Arc ensures data lives as long as needed
        chunks
            .par_iter()
            .flat_map(|chunk| {
                // chunk is Arc<Chunk>, cloned cheaply
                self.match_chunk(Arc::clone(chunk))
            })
            .collect()
    }
}

// For circular references, use Weak:
use std::sync::Weak;

pub struct Node {
    parent: Weak<Node>,  // Weak to avoid cycles
    children: Vec<Arc<Node>>,
}
```

**Compatibility Impact**:
| Aspect | Impact | Mitigation |
|--------|--------|------------|
| Memory usage | Lower in Rust (no GC overhead) | Measure and compare |
| Compile errors | Ownership errors | Learn Rust patterns |
| Runtime errors | Fewer (compile-time catches) | Handle edge cases |

**Regression Verification**:
- Memory leak detection: `valgrind` or `miri`
- Long-running tests (hours/days)
- Memory profiling comparison

---

### 2.2 Slab Allocator Implementation

| Aspect | Go | Rust Challenge | Alternative |
|--------|-----|----------------|-------------|
| **Manual slab** | Custom implementation | Ownership complexity | typed-arena crate |
| **Reuse pattern** | Reset and reuse | Lifetime constraints | bumpalo for bump alloc |

**Go Code**:
```go
// Go: Simple slab with GC
type Slab struct {
    data []int32
}

func (s *Slab) Reset() {
    s.data = s.data[:0]  // Reset slice, GC handles old memory
}
```

**Rust Alternative**:
```rust
// Strategy: bumpalo for bump allocation
use bumpalo::Bump;

pub struct SlabAllocator {
    arena: Bump,
}

impl SlabAllocator {
    pub fn allocate<T>(&self, val: T) -> &mut T {
        self.arena.alloc(val)
    }

    pub fn reset(&mut self) {
        self.arena.reset();  // O(1) reset
    }
}

// For type-specific slabs:
pub struct IntSlab {
    storage: Vec<i32>,
    in_use: Vec<bool>,
}

impl IntSlab {
    pub fn alloc(&mut self, val: i32) -> usize {
        if let Some(idx) = self.in_use.iter().position(|&u| !u) {
            self.storage[idx] = val;
            self.in_use[idx] = true;
            idx
        } else {
            let idx = self.storage.len();
            self.storage.push(val);
            self.in_use.push(true);
            idx
        }
    }

    pub fn reset(&mut self) {
        self.in_use.fill(false);
    }
}
```

**Compatibility Impact**:
| Aspect | Impact | Mitigation |
|--------|--------|------------|
| Performance | Better in Rust (no GC) | Benchmark comparison |
| Safety | Compile-time | Use safe abstractions |
| API change | Different interface | Wrapper or port |

**Regression Verification**:
```rust
#[test]
fn test_slab_allocator_correctness() {
    let mut slab = IntSlab::new();

    let a = slab.alloc(1);
    let b = slab.alloc(2);

    assert_eq!(slab.get(a), 1);
    assert_eq!(slab.get(b), 2);

    slab.reset();

    let c = slab.alloc(3);
    assert_eq!(c, a); // Should reuse slot
}
```

---

## 3. Type System

### 3.1 Interface Type Assertions vs Rust Enums

| Aspect | Go | Rust Challenge | Alternative |
|--------|-----|----------------|-------------|
| **Runtime types** | Interface{} + type switch | No runtime reflection | Enum variants with data |
| **Type assertions** | `x.(Type)` | Pattern matching | `match` with enum variants |

**Go Code**:
```go
// Go: Type switch on interface{}
type EventValue interface{}

type MatchRequest struct { /* ... */ }

func (m *Matcher) handleRequest(val EventValue) {
    switch v := val.(type) {
    case MatchRequest:
        m.scan(v)
    case string:
        m.cancel()
    default:
        panic(fmt.Sprintf("unexpected: %T", val))
    }
}
```

**Rust Alternative**:
```rust
// Strategy: Enum variants with associated data
pub enum EventValue {
    MatchRequest(MatchRequest),
    Cancel(String),
    Quit,
}

impl Matcher {
    pub fn handle_request(&mut self, val: EventValue) {
        match val {
            EventValue::MatchRequest(req) => {
                self.scan(&req);
            }
            EventValue::Cancel(reason) => {
                info!("Cancelling: {}", reason);
                self.cancel();
            }
            EventValue::Quit => {
                self.shutdown();
            }
        }
    }
}

// For truly dynamic cases, use Any trait
use std::any::Any;

pub fn handle_dynamic(val: Box<dyn Any>) {
    if let Some(req) = val.downcast_ref::<MatchRequest>() {
        // Handle MatchRequest
    } else if let Some(s) = val.downcast_ref::<String>() {
        // Handle String
    } else {
        panic!("unexpected type");
    }
}
```

**Compatibility Impact**:
| Aspect | Impact | Mitigation |
|--------|--------|------------|
| Performance | Better in Rust (no reflection) | Enum dispatch is fast |
| Type safety | Compile-time | Catches errors early |
| Flexibility | Less dynamic | Design around enums |

**Regression Verification**:
- All event types handled in match
- No unreachable patterns (compiler catches)

---

### 3.2 Reflection vs Compile-Time Code Generation

| Aspect | Go | Rust Challenge | Alternative |
|--------|-----|----------------|-------------|
| **reflect package** | Runtime type inspection | Limited | proc-macro for derive |
| **JSON marshaling** | `json:"field"` tags | Compile-time | serde derive macros |
| **Stringer** | `go generate` | Compile-time | strum or custom macro |

**Go Code**:
```go
// Go: Runtime reflection for JSON
type Options struct {
    Filter string `json:"filter,omitempty"`
    Exact  bool   `json:"exact"`
}

// Stringer via go generate
//go:generate stringer -type=EventType
type EventType int
```

**Rust Alternative**:
```rust
// Strategy: Serde for serialization
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Options {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,
    pub exact: bool,
}

// String representation via strum or custom
use strum::{Display, EnumString};

#[derive(Display, EnumString)]
pub enum EventType {
    #[strum(serialize = "read_new")]
    ReadNew,
    #[strum(serialize = "read_fin")]
    ReadFin,
}

// Or manual impl for complex cases
pub enum EventType {
    ReadNew,
    ReadFin,
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            EventType::ReadNew => write!(f, "read_new"),
            EventType::ReadFin => write!(f, "read_fin"),
        }
    }
}
```

**Compatibility Impact**:
| Aspect | Impact | Mitigation |
|--------|--------|------------|
| Binary size | Smaller (no reflection metadata) | Acceptable |
| Compilation | Slower (macros) | Incremental builds |
| Performance | Faster (no runtime reflection) | Benefit |

**Regression Verification**:
- JSON round-trip tests
- Serde compatibility with Go output

---

## 4. Error Handling

### 4.1 Panic/Recover vs Panic Handling

| Aspect | Go | Rust Challenge | Alternative |
|--------|-----|----------------|-------------|
| **panic** | Runtime panic | Unwinding or abort | `panic!` with catch_unwind |
| **recover** | defer + recover | Limited | `catch_unwind` for isolation |
| **Usage** | Expected errors vs bugs | Strict separation | Result for errors, panic for bugs |

**Go Code**:
```go
// Go: Using panic/recover for error handling
func (m *Matcher) Loop() {
    defer func() {
        if r := recover(); r != nil {
            log.Printf("Matcher panic: %v", r)
        }
    }()

    for {
        // Work that might panic
    }
}
```

**Rust Alternative**:
```rust
// Strategy 1: catch_unwind for isolation (rarely needed)
use std::panic::catch_unwind;

impl Matcher {
    pub fn run_isolated(&self) -> Result<(), Box<dyn Any + Send>> {
        catch_unwind(|| {
            self.run()
        })
    }
}

// Strategy 2: Proper error handling (preferred)
impl Matcher {
    pub fn run(&self) -> Result<(), MatcherError> {
        loop {
            // Return Err instead of panic
            if let Err(e) = self.do_work() {
                error!("Work failed: {}", e);
                return Err(e.into());
            }
        }
    }
}

// Strategy 3: Abort on panic (for simplicity)
// In Cargo.toml:
// [profile.release]
// panic = "abort"
```

**Compatibility Impact**:
| Aspect | Impact | Mitigation |
|--------|--------|------------|
| Behavior | Rust panics are bugs | Never catch expected errors |
| Safety | Rust safer | Use Result everywhere |
| Performance | Unwinding has cost | Use `panic = "abort"` if acceptable |

**Regression Verification**:
- No panics in release builds (test with release mode)
- Fuzz testing to find panic cases

---

### 4.2 Error Wrapping

| Aspect | Go | Rust Challenge | Alternative |
|--------|-----|----------------|-------------|
| **Error wrapping** | `fmt.Errorf("... %w", err)` | Context attachment | `anyhow::Context` |
| **Error inspection** | `errors.Is/As` | Downcasting | `downcast_ref` |

**Go Code**:
```go
// Go: Error wrapping and inspection
err := doSomething()
if err != nil {
    return fmt.Errorf("failed to do something: %w", err)
}

// Later:
if errors.Is(err, ErrNotFound) {
    // Handle not found
}
```

**Rust Alternative**:
```rust
// Strategy: anyhow for application code
use anyhow::{Context, Result};

fn do_something() -> Result<()> {
    do_inner().context("failed to do something")?
}

// Inspection:
if let Some(e) = err.downcast_ref::<io::Error>() {
    if e.kind() == io::ErrorKind::NotFound {
        // Handle not found
    }
}

// For library code, use thiserror:
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MatcherError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}
```

**Compatibility Impact**:
| Aspect | Impact | Mitigation |
|--------|--------|------------|
| Error messages | May differ | Ensure key info preserved |
| Programmatic checks | Different API | Adapt error handling |

**Regression Verification**:
- Error message contains original error
- Error chains preserved

---

## 5. Runtime Features

### 5.1 Defer vs Drop/Scope Guards

| Aspect | Go | Rust Challenge | Alternative |
|--------|-----|----------------|-------------|
| **defer** | Stack-based cleanup | No defer keyword | RAII + Drop trait |
| **Multiple defers** | LIFO order | Scoped drops | Scope guards |

**Go Code**:
```go
// Go: Multiple defers (LIFO)
func process() error {
    f, err := os.Open("file")
    if err != nil {
        return err
    }
    defer f.Close()  // Executes last

    temp := createTemp()
    defer os.Remove(temp)  // Executes first

    // Do work...
    return nil
}
```

**Rust Alternative**:
```rust
// Strategy 1: RAII with Drop
struct TempFile {
    path: PathBuf,
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

impl TempFile {
    fn create() -> io::Result<Self> {
        let path = create_temp()?;
        Ok(Self { path })
    }
}

// Strategy 2: scopeguard crate
use scopeguard::defer;

fn process() -> anyhow::Result<()> {
    let f = File::open("file")?;
    defer! {
        drop(f); // Close file
    }

    let temp = create_temp()?;
    defer! {
        let _ = std::fs::remove_file(&temp);
    }

    // Do work...
    Ok(())
}

// Strategy 3: explicit try/finally pattern
fn process() -> anyhow::Result<()> {
    let f = File::open("file")?;

    let result = (|| {
        // Do work...
        Ok(())
    })();

    drop(f); // Explicit cleanup
    result
}
```

**Compatibility Impact**:
| Aspect | Impact | Mitigation |
|--------|--------|------------|
| Cleanup timing | Drop at end of scope | Similar to defer |
| Order | Not LIFO by default | Use scopeguard or explicit |
| Exceptions | Drop always runs | Unwinding runs Drop |

**Regression Verification**:
```rust
#[test]
fn test_cleanup_runs() {
    use std::sync::atomic::{AtomicBool, Ordering};

    let cleaned = AtomicBool::new(false);

    {
        let guard = scopeguard::guard((), |_| {
            cleaned.store(true, Ordering::SeqCst);
        });
        // guard dropped here
    }

    assert!(cleaned.load(Ordering::SeqCst));
}
```

---

### 5.2 CGO vs FFI

| Aspect | Go | Rust Challenge | Alternative |
|--------|-----|----------------|-------------|
| **C interop** | cgo | FFI | bindgen + cc crate |
| **Overhead** | High (context switch) | Low | Direct calls |

**Go Code**:
```go
// Go: CGO for SIMD
// #include "simd_amd64.h"
import "C"

func simdSearch(data []byte, target byte) int {
    return int(C.indexbyte((*C.char)(unsafe.Pointer(&data[0])),
                            C.size_t(len(data)),
                            C.char(target)))
}
```

**Rust Alternative**:
```rust
// Strategy 1: Inline assembly (nightly)
#[cfg(target_arch = "x86_64")]
pub unsafe fn simd_search(data: &[u8], target: u8) -> Option<usize> {
    use std::arch::x86_64::*;

    let target_vec = _mm_set1_epi8(target as i8);
    let ptr = data.as_ptr();

    for i in (0..data.len()).step_by(16) {
        let chunk = _mm_loadu_si128(ptr.add(i) as *const _);
        let cmp = _mm_cmpeq_epi8(chunk, target_vec);
        let mask = _mm_movemask_epi8(cmp);

        if mask != 0 {
            return Some(i + mask.trailing_zeros() as usize);
        }
    }

    None
}

// Strategy 2: C FFI with bindgen
#[link(name = "simd")]
extern "C" {
    fn indexbyte(data: *const u8, len: usize, target: u8) -> isize;
}

pub fn simd_search(data: &[u8], target: u8) -> Option<usize> {
    let result = unsafe { indexbyte(data.as_ptr(), data.len(), target) };
    if result >= 0 {
        Some(result as usize)
    } else {
        None
    }
}
```

**Compatibility Impact**:
| Aspect | Impact | Mitigation |
|--------|--------|------------|
| Performance | Faster (no cgo overhead) | Benchmark gain |
| Safety | unsafe blocks required | Careful auditing |
| Portability | Conditional compilation | cfg flags |

**Regression Verification**:
```rust
#[test]
fn test_simd_equivalent_to_scalar() {
    let data = b"hello world this is a test";

    for target in b"abcdefghijklmnopqrstuvwxyz" {
        let simd_result = unsafe { simd_search(data, *target) };
        let scalar_result = data.iter().position(|&b| b == *target);

        assert_eq!(simd_result, scalar_result,
                   "Mismatch for target '{}'", *target as char);
    }
}
```

---

## 6. Summary Table

| Go Feature | Rust Alternative | Compatibility Risk | Verification |
|------------|------------------|-------------------|--------------|
| Goroutines | tokio + rayon | Low | Parallel equivalence tests |
| Select | tokio::select! | Low | Event ordering tests |
| sync.Cond | Notify/Watch | Low | Concurrency stress tests |
| GC | Ownership + Arc | Low | Memory leak tests |
| Slab allocator | bumpalo | Low | Correctness tests |
| Interface{} | Enum/Any | Medium | Type coverage tests |
| Reflection | Serde/Macro | Low | JSON round-trip tests |
| Panic/Recover | catch_unwind | Medium | No panic in release |
| Error wrapping | anyhow | Low | Error chain tests |
| Defer | Drop/scopeguard | Low | Cleanup verification |
| CGO | FFI/bindgen | Medium | SIMD correctness tests |

---

## 7. Regression Verification Requirements

### 7.1 Per-Alternative Verification

| Alternative | Required Tests | Acceptance Criteria |
|-------------|----------------|---------------------|
| tokio async | Concurrent event handling | No deadlocks, events processed |
| rayon parallel | Parallel matching | Results match sequential |
| Arc sharing | Shared state access | No data races (Miri) |
| Enum dispatch | Type handling | All variants handled |
| FFI calls | SIMD/assembly | Output matches pure Rust |
| Drop cleanup | Resource cleanup | No leaks (valgrind) |

### 7.2 Continuous Verification

```yaml
# CI verification
- cargo test --release
- cargo miri test  # Undefined behavior detection
- cargo fuzz  # Panic detection
- valgrind --leak-check=full ./fzf
```
