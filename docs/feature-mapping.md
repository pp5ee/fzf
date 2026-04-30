# fzf Go→Rust Feature Mapping Strategy

## Overview

This document provides detailed mapping strategies for translating Go features to Rust equivalents, covering concurrency, error handling, data structures, serialization, and logging.

---

## 1. Concurrency Model Mapping

### 1.1 Go Concurrency Primitives

| Go Feature | Purpose | Usage in fzf |
|------------|---------|--------------|
| `goroutine` | Lightweight thread | Matcher workers, event polling, reader |
| `chan` | Typed message passing | EventBox internal (via `sync.Cond`) |
| `sync.Cond` | Condition variable | EventBox.Wait/Broadcast |
| `sync.Mutex` | Mutual exclusion | ChunkList, EventBox |
| `sync.RWMutex` | Read-preference lock | Cache access |
| `sync.WaitGroup` | Goroutine synchronization | Parallel matching |
| `context.Context` | Cancellation/timeout | Command execution |
| `atomic` | Lock-free operations | Cancellation flags, counters |

### 1.2 Rust Equivalent Strategy

```rust
// Strategy: Hybrid tokio + rayon + parking_lot

// 1. Async I/O and event loop: tokio
use tokio::sync::{mpsc, watch, RwLock};
use tokio::task;
use tokio::runtime::Runtime;

// 2. CPU-bound parallelism: rayon
use rayon::prelude::*;
use rayon::ThreadPool;

// 3. Low-level synchronization: parking_lot
use parking_lot::{Mutex, RwLock, Condvar};

// 4. Lock-free operations: std::sync::atomic
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
```

### 1.3 Module-Specific Concurrency Mapping

#### Core Event Loop (`core.rs`)

```go
// Go: EventBox with sync.Cond
eventBox := util.NewEventBox()
eventBox.Wait(func(events *util.Events) {
    // Process events
})
```

```rust
// Rust Strategy: tokio::sync::mpsc + watch channels
use tokio::sync::{mpsc, watch};

// Event types
#[derive(Debug, Clone)]
pub enum Event {
    ReadNew,
    ReadFin,
    SearchNew(String),
    SearchProgress(f32),
    SearchFin(Vec<Result>),
    Quit,
}

// Event channels
pub struct EventBus {
    // For one-to-many broadcasting
    pub tx: watch::Sender<Event>,
    pub rx: watch::Receiver<Event>,
    // For request-response
    pub req_tx: mpsc::Sender<MatchRequest>,
    pub req_rx: mpsc::Receiver<MatchRequest>,
}

impl EventBus {
    pub async fn run_event_loop(&mut self) -> Result<i32> {
        loop {
            tokio::select! {
                Some(event) = self.rx.recv() => {
                    match event {
                        Event::Quit => return Ok(EXIT_OK),
                        _ => self.handle_event(event).await?,
                    }
                }
                Some(req) = self.req_rx.recv() => {
                    self.handle_request(req).await?;
                }
            }
        }
    }
}
```

#### Matcher Parallelism (`matcher.rs`)

```go
// Go: Goroutines with sync.WaitGroup
func (m *Matcher) scan(request MatchRequest) MatchResult {
    // Partition work among workers
    for i := 0; i < m.partitions; i++ {
        go m.worker(i, chunks[i], results[i])
    }
    // Collect results
}
```

```rust
// Rust Strategy: rayon parallel iterator
use rayon::prelude::*;

impl Matcher {
    pub fn scan_parallel(&self, chunks: &[Arc<Chunk>], pattern: &Pattern) -> Vec<Result> {
        // Automatic work-stealing parallelism
        chunks
            .par_iter()  // Parallel iterator
            .flat_map(|chunk| self.match_chunk(chunk, pattern))
            .collect()
    }

    // For more control, use ThreadPool
    pub fn scan_with_pool(&self, pool: &ThreadPool, chunks: &[Arc<Chunk>]) -> Vec<Result> {
        pool.install(|| {
            chunks
                .par_iter()
                .flat_map(|chunk| self.match_chunk(chunk, pattern))
                .collect()
        })
    }
}
```

#### Reader Async I/O (`reader.rs`)

```go
// Go: Goroutine with polling
func (r *Reader) startEventPoller() {
    go func() {
        for {
            if atomic.CompareAndSwapInt32(&r.event, EvtReadNew, EvtReady) {
                r.eventBox.Set(EvtReadNew, nil)
            }
            time.Sleep(pollInterval)
        }
    }()
}
```

```rust
// Rust Strategy: tokio async with channels
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};

pub struct Reader {
    event_tx: mpsc::Sender<Event>,
}

impl Reader {
    pub async fn start_event_poller(&self) {
        let mut interval = interval(Duration::from_millis(10));
        let event_tx = self.event_tx.clone();

        tokio::spawn(async move {
            loop {
                interval.tick().await;
                if should_notify() {
                    let _ = event_tx.send(Event::ReadNew).await;
                }
            }
        });
    }

    pub async fn read_stdin_async(&self) -> io::Result<()> {
        use tokio::io::AsyncBufReadExt;

        let stdin = tokio::io::stdin();
        let mut lines = stdin.lines();

        while let Some(line) = lines.next_line().await? {
            self.process_line(line).await?;
        }

        self.event_tx.send(Event::ReadFin).await.ok();
        Ok(())
    }
}
```

### 1.4 Concurrency Mapping Summary

| Go Pattern | Rust Pattern | Crate | When to Use |
|------------|--------------|-------|-------------|
| `goroutine` | `tokio::spawn` | tokio | I/O-bound async tasks |
| `goroutine` | `rayon::spawn` | rayon | CPU-bound parallel tasks |
| `chan` | `mpsc::channel` | tokio | Many-to-one message passing |
| `chan` | `watch::channel` | tokio | One-to-many broadcasting |
| `sync.Cond` | `tokio::sync::Notify` | tokio | Event notification |
| `sync.Mutex` | `parking_lot::Mutex` | parking_lot | Sync mutex (faster than std) |
| `sync.RWMutex` | `parking_lot::RwLock` | parking_lot | Read-heavy access |
| `sync.WaitGroup` | `tokio::task::JoinSet` | tokio | Wait for multiple tasks |
| `context.Context` | `tokio_util::sync::CancellationToken` | tokio-util | Cancellation |
| `atomic` | `std::sync::atomic` | std | Lock-free operations |

---

## 2. Error Handling Mapping

### 2.1 Go Error Handling

```go
// Go: error interface + multiple returns
func Run(opts *Options) (int, error) {
    terminal, err := NewTerminal(opts, eventBox, executor)
    if err != nil {
        return ExitError, err  // Early return
    }
    // ...
}

// Custom error types
type PatternError struct {
    Msg string
    Pos int
}

func (e *PatternError) Error() string {
    return fmt.Sprintf("pattern error at %d: %s", e.Pos, e.Msg)
}

// Error wrapping (Go 1.13+)
return fmt.Errorf("failed to parse: %w", err)
```

### 2.2 Rust Error Handling Strategy

```rust
// Strategy: anyhow for application, thiserror for libraries

// Application errors (main.rs, core.rs)
use anyhow::{Result, Context, bail};

// Library errors (pattern.rs, matcher.rs)
use thiserror::Error;

// Custom error types for specific modules
#[derive(Error, Debug)]
pub enum PatternError {
    #[error("invalid pattern at position {pos}: {msg}")]
    InvalidSyntax { pos: usize, msg: String },

    #[error("unsupported regex: {0}")]
    InvalidRegex(#[from] regex::Error),

    #[error("empty pattern")]
    EmptyPattern,
}

// Application-level error handling
pub fn run(opts: Options) -> Result<i32> {
    // anyhow::Context for error messages
    let terminal = Terminal::new(opts.clone())
        .context("failed to initialize terminal")?;

    // anyhow::bail for early returns with errors
    if opts.filter.is_none() && !stdin().is_terminal() {
        bail!("terminal required for interactive mode");
    }

    // Automatic error conversion with ?
    terminal.run()?;

    Ok(EXIT_OK)
}
```

### 2.3 Error Handling Mapping Table

| Go Pattern | Rust Pattern | Example |
|------------|--------------|---------|
| `if err != nil { return err }` | `?` operator | `let x = fallible()?;` |
| `fmt.Errorf("... %w", err)` | `.context("...")` | `.context("reading file")?` |
| Custom error struct | `#[derive(Error)]` | `#[error("...")]` |
| `errors.Is(err, target)` | `downcast_ref()` | `if let Some(e) = err.downcast_ref::<PatternError>()` |
| `errors.As(err, &target)` | Downcasting | `err.downcast::<io::Error>()` |
| `panic()` | `panic!()` or `unreachable!()` | For programming errors only |
| `log.Fatal()` | `error!()` + exit | Log then return error |

### 2.4 Module-Specific Error Types

```rust
// src/pattern.rs
#[derive(Error, Debug)]
pub enum PatternError {
    #[error("invalid syntax at {pos}: {message}")]
    SyntaxError { pos: usize, message: String },
}

// src/options.rs
#[derive(Error, Debug)]
pub enum OptionsError {
    #[error("unknown option: {0}")]
    UnknownOption(String),

    #[error("invalid value for {option}: {value}")]
    InvalidValue { option: String, value: String },
}

// src/terminal.rs
#[derive(Error, Debug)]
pub enum TerminalError {
    #[error("terminal initialization failed: {0}")]
    InitError(String),

    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error(" Crossterm error: {0}")]
    Crossterm(#[from] crossterm::ErrorKind),
}

// src/lib.rs - unify error types
pub type Result<T> = anyhow::Result<T>;
```

---

## 3. Data Structures & Ownership Mapping

### 3.1 Go vs Rust Memory Model

| Aspect | Go | Rust |
|--------|-----|------|
| Memory management | Garbage collector | Ownership + borrowing |
| Shared state | Pointers (GC-safe) | `Arc<T>` + `Mutex<T>` |
| Mutability | Always mutable | `mut` keyword required |
| Null values | `nil` | `Option<T>` |
| Slices | `[]T` (growable view) | `&[T]` or `Vec<T>` |
| Maps | `map[K]V` | `HashMap<K, V>` |

### 3.2 Key Data Structure Mappings

#### Item / Chars

```go
// Go: GC-managed, pointer-based
type Item struct {
    text     util.Chars
    origText *[]byte
    colors   *[]ansiOffset
    index    int32
}
```

```rust
// Rust: Ownership-based with Arc for sharing
use std::sync::Arc;

#[derive(Clone)]
pub struct Item {
    // Arc for cheap cloning when sharing between threads
    pub text: Arc<Chars>,
    pub colors: Option<Arc<Vec<AnsiOffset>>>,
    pub index: u32,
}

// Chars - immutable after creation
#[derive(Clone)]
pub struct Chars {
    data: Vec<char>,
    index: u32,
}

impl Chars {
    // Borrow instead of copy
    pub fn as_slice(&self) -> &[char] {
        &self.data
    }

    // Cheap clone via Arc
    pub fn to_shared(self) -> Arc<Self> {
        Arc::new(self)
    }
}
```

#### ChunkList (Shared State)

```go
// Go: Mutex-protected shared state
type ChunkList struct {
    cache    *ChunkCache
    chunks   []*Chunk
    mutex    sync.Mutex
}
```

```rust
// Rust: Arc<Mutex<T>> for thread-safe sharing
use std::sync::{Arc, Mutex};
use parking_lot::Mutex as FastMutex; // Faster than std

pub struct ChunkList {
    cache: ChunkCache,
    chunks: Vec<Chunk>,
}

// Thread-safe wrapper
pub type SharedChunkList = Arc<FastMutex<ChunkList>>;

// Usage in Reader/Matcher
pub struct Reader {
    chunk_list: SharedChunkList,
}

pub struct Matcher {
    chunk_list: SharedChunkList,  // Shared with Reader
}
```

#### Pattern (Immutable Data)

```go
// Go: Mutable struct with methods
type Pattern struct {
    text      []rune
    fuzzy     bool
    caseMode  Case
    // ... mutable fields
}

func (p *Pattern) Match(text string) (Result, bool) {
    // ...
}
```

```rust
// Rust: Immutable struct, cheap to clone
#[derive(Clone)]
pub struct Pattern {
    text: Vec<char>,  // char = Unicode scalar (like Go rune)
    fuzzy: bool,
    case_mode: Case,
}

impl Pattern {
    // &self = immutable borrow (no mutation)
    pub fn match_text(&self, text: &[char]) -> Option<MatchResult> {
        // Pattern is immutable during matching
    }
}

// Arc<Pattern> for sharing across threads
pub type SharedPattern = Arc<Pattern>;
```

### 3.3 Ownership Patterns

```rust
// Pattern 1: Move semantics (transfer ownership)
let item = Item::new(text);
process_item(item);  // item moved, no longer usable

// Pattern 2: Borrowing (temporary access)
let item = Item::new(text);
process_item_ref(&item);  // borrowed, still usable
println!("{:?}", item);   // OK

// Pattern 3: Shared ownership (Arc)
let item = Arc::new(Item::new(text));
let item2 = Arc::clone(&item);  // cheap clone
process_item_async(item);   // moved into async task
process_item_async(item2);  // moved into another task

// Pattern 4: Interior mutability (Mutex)
let counter = Arc::new(Mutex::new(0));
let counter2 = Arc::clone(&counter);

// Thread 1
*counter.lock() += 1;

// Thread 2
*counter2.lock() += 1;

// Pattern 5: Copy-on-Write (Cow)
use std::borrow::Cow;

pub fn process_text(text: &str) -> Cow<str> {
    if needs_modification(text) {
        Cow::Owned(text.to_uppercase())  // allocate if modified
    } else {
        Cow::Borrowed(text)  // zero-cost if not modified
    }
}
```

### 3.4 Lifetime Management

```rust
// Go: GC handles lifetimes automatically
// Rust: Explicit lifetimes for borrowed data

// Pattern: 'static for data that lives forever
pub struct Matcher {
    // 'static means pattern can live for program lifetime
    pattern: Arc<Pattern>,  // Arc gives 'static lifetime
}

// Pattern: Scoped lifetimes
pub struct Context<'a> {
    // References to external data
    options: &'a Options,
    terminal: &'a mut Terminal,
}

impl<'a> Context<'a> {
    // Lifetime 'a ensures data outlives context
    pub fn run(&mut self) -> Result<()> {
        self.terminal.draw(&self.options)?;
        Ok(())
    }
}
```

---

## 4. Serialization Protocol Mapping

### 4.1 Go JSON Handling

```go
// Go: encoding/json with struct tags
type Action struct {
    Type    string   `json:"type"`
    Key     string   `json:"key,omitempty"`
    Args    []string `json:"args,omitempty"`
}

// Manual unmarshaling for complex cases
func ParseAction(data []byte) (*Action, error) {
    var raw map[string]interface{}
    if err := json.Unmarshal(data, &raw); err != nil {
        return nil, err
    }
    // Custom parsing logic...
}
```

### 4.2 Rust Serialization Strategy

```rust
// Strategy: serde for JSON serialization
use serde::{Serialize, Deserialize};

// Server API types (src/server.rs)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    #[serde(rename = "execute")]
    Execute { command: String },
    #[serde(rename = "abort")]
    Abort,
    #[serde(rename = "accept")]
    Accept { output: Option<String> },
}

// Request/Response types
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiRequest {
    pub action: Action,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub results: Option<Vec<String>>,
}

// Options serialization for history/state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentOptions {
    pub query: String,
    pub selected: Vec<usize>,
    #[serde(default)]
    pub scroll: usize,
}
```

### 4.3 Serialization Mapping Table

| Go | Rust | serde Attribute | Purpose |
|----|------|-----------------|---------|
| `json:"field"` | `#[serde(rename = "field")]` | Rename field |
| `json:"-"` | `#[serde(skip)]` | Skip field |
| `json:"field,omitempty"` | `#[serde(skip_serializing_if = "Option::is_none")]` | Skip if empty |
| `interface{}` | `serde_json::Value` | Dynamic JSON |
| Custom `UnmarshalJSON` | `Deserialize` impl | Custom parsing |
| Custom `MarshalJSON` | `Serialize` impl | Custom output |

### 4.4 Server Mode Serialization

```rust
// src/server.rs - HTTP API serialization
use axum::{
    routing::post,
    Json,
    Router,
};

async fn handle_action(
    Json(request): Json<ApiRequest>,
) -> Json<ApiResponse> {
    let result = match request.action {
        Action::Execute { command } => {
            match execute_command(&command).await {
                Ok(output) => ApiResponse {
                    success: true,
                    error: None,
                    results: Some(vec![output]),
                },
                Err(e) => ApiResponse {
                    success: false,
                    error: Some(e.to_string()),
                    results: None,
                },
            }
        }
        Action::Abort => {
            // Signal abort to event loop
            ApiResponse { success: true, .. }
        }
        Action::Accept { output } => {
            // Handle selection
            ApiResponse { success: true, .. }
        }
    };

    Json(result)
}

// Streaming JSON lines (for results)
pub fn serialize_results(results: &[Result]) -> impl Iterator<Item = String> + '_ {
    results.iter().map(|r| {
        serde_json::json!({
            "text": r.item.text(),
            "score": r.rank.score,
            "index": r.rank.index,
        }).to_string()
    })
}
```

---

## 5. Configuration & Logging Mapping

### 5.1 Go Configuration

```go
// Go: Options struct + flag parsing
type Options struct {
    Filter    *string
    Exact     bool
    CaseMode  Case
    // ... many fields
}

// Manual validation
func postProcessOptions(opts *Options) error {
    if opts.MinHeight < 1 {
        return errors.New("min-height must be >= 1")
    }
    // ...
}
```

### 5.2 Rust Configuration Strategy

```rust
// Strategy: clap for CLI + config-rs for files

// src/options.rs
use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(name = "fzf")]
#[command(about = "A command-line fuzzy finder")]
pub struct Options {
    /// Filter mode (non-interactive)
    #[arg(short, long)]
    pub filter: Option<String>,

    /// Enable exact match
    #[arg(short = 'e', long)]
    pub exact: bool,

    /// Case sensitivity mode
    #[arg(long, value_enum, default_value = "smart")]
    pub case: CaseMode,

    /// Minimum height
    #[arg(long, value_name = "HEIGHT")]
    pub min_height: Option<usize>,

    /// Key bindings
    #[arg(long = "bind", value_name = "KEY:ACTION")]
    pub binds: Vec<String>,
}

#[derive(Debug, Clone, Copy, Default, ValueEnum)]
pub enum CaseMode {
    #[default]
    Smart,
    Ignore,
    Respect,
}

// Validation with clap
impl Options {
    pub fn validate(&self) -> anyhow::Result<()> {
        if let Some(h) = self.min_height {
            if h < 1 {
                return Err(anyhow!("min-height must be >= 1"));
            }
        }
        Ok(())
    }
}
```

### 5.3 Logging Mapping

```go
// Go: No standard logging, manual or external
// In fzf: mostly silent, some debug output
```

```rust
// Rust: tracing for structured logging
use tracing::{info, debug, warn, error, span, Level};

// Initialize in main
fn main() {
    tracing_subscriber::fmt::init();

    // Or with more control:
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(false)
        .init();
}

// Usage throughout codebase
pub fn run(opts: Options) -> Result<i32> {
    let span = span!(Level::INFO, "fzf_run", filter = ?opts.filter);
    let _enter = span.enter();

    debug!(options = ?opts, "starting fzf");

    let reader = Reader::new();
    info!("reader initialized");

    match reader.read_stdin() {
        Ok(items) => {
            debug!(count = items.len(), "items loaded");
        }
        Err(e) => {
            error!(error = %e, "failed to read input");
            return Err(e.into());
        }
    }

    // Conditional compilation for debug builds
    #[cfg(debug_assertions)]
    {
        trace!("detailed state: {:?}", state);
    }

    Ok(EXIT_OK)
}

// Instrument async functions
#[tracing::instrument(skip(self))]
pub async fn match_async(&self, pattern: &Pattern) -> Vec<Result> {
    debug!(pattern = %pattern.text(), "starting match");
    // ...
}

// Structured fields
info!(
    target: "matcher",
    pattern = %pattern,
    chunks = chunks.len(),
    partitions = self.partitions,
    "matching started"
);
```

### 5.4 Configuration File Support (Optional)

```rust
// Strategy: config-rs for TOML/JSON/YAML configs
use config::{Config, File, Environment};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ConfigFile {
    pub key_bindings: Option<Vec<KeyBinding>>,
    pub default_options: Option<Options>,
    pub color_scheme: Option<String>,
}

impl Options {
    pub fn from_config(path: &str) -> anyhow::Result<Self> {
        let settings = Config::builder()
            .add_source(File::with_name(path))
            .add_source(Environment::with_prefix("FZF"))
            .build()?;

        let config: ConfigFile = settings.try_deserialize()?;
        // Merge with CLI options...
        Ok(Self::from(config))
    }
}
```

---

## 6. Feature Mapping Summary Table

| Category | Go | Rust | Crate | Notes |
|----------|-----|------|-------|-------|
| **Async I/O** | Goroutine + chan | tokio::spawn + mpsc | tokio | Similar performance |
| **Parallelism** | sync.WaitGroup | rayon::join | rayon | Better work-stealing |
| **Mutex** | sync.Mutex | parking_lot::Mutex | parking_lot | Faster, no poisoning |
| **Error** | error interface | anyhow::Result | anyhow | Ergonomic |
| **Custom Error** | struct + Error() | #[derive(Error)] | thiserror | Derive macro |
| **Shared State** | Pointers | Arc<Mutex<T>> | std | Explicit ownership |
| **JSON** | encoding/json | serde::Serialize | serde | Compile-time |
| **CLI Args** | flag package | clap::Parser | clap | Derive macros |
| **Logging** | log package | tracing | tracing | Structured |
| **Config File** | Manual | config | config-rs | Multi-format |
| **Channels** | chan T | mpsc::channel<T> | tokio | Async-aware |
| **Atomics** | sync/atomic | std::sync::atomic | std | Similar API |
| **Slices** | []T | &[T] / Vec<T> | std | Explicit mut |
| **Maps** | map[K]V | HashMap<K,V> | std | No nil values |

---

## 7. Implementation Guidelines

### 7.1 Concurrency Rules

1. **Use tokio for I/O, rayon for CPU**
   - Reader: tokio async
   - Matcher: rayon parallel
   - Event loop: tokio::select!

2. **Avoid blocking in async contexts**
   ```rust
   // BAD: Blocks async runtime
   tokio::spawn(async {
       heavy_computation();  // Blocks!
   });

   // GOOD: Spawn on rayon
   tokio::spawn(async {
       tokio::task::spawn_blocking(|| {
           heavy_computation()
       }).await;
   });
   ```

3. **Prefer channels over shared state**
   - Event passing: mpsc/watch
   - State updates: message to owner

### 7.2 Error Handling Rules

1. **Use `anyhow` in application code**
   - main.rs, core.rs
   - Context for error messages

2. **Use `thiserror` in library modules**
   - pattern.rs, matcher.rs
   - Public error types

3. **Never unwrap in production code**
   ```rust
   // Use expect with reason
   let val = opt.expect("invariant: opt is Some after check");

   // Or propagate
   let val = opt.ok_or_else(|| anyhow!("opt was None"))?;
   ```

### 7.3 Ownership Rules

1. **Clone is cheap with Arc**
   ```rust
   let pattern = Arc::new(pattern);
   let pattern2 = Arc::clone(&pattern);  // O(1)
   ```

2. **Borrow for read-only access**
   ```rust
   fn process(items: &[Item]) {  // Borrow
       for item in items { }
   }
   ```

3. **Mutex guard scope**
   ```rust
   {
       let mut guard = list.lock();
       guard.push(item);
   }  // Drop lock here
   // Other work without holding lock
   ```

### 7.4 Serialization Rules

1. **Derive by default**
   ```rust
   #[derive(Serialize, Deserialize)]
   struct Data { }
   ```

2. **Custom impl for complex cases**
   - Manual `Deserialize` for pattern parsing
   - Custom `Serialize` for terminal output

3. **Version your APIs**
   ```rust
   #[serde(tag = "version")]
   enum ApiRequest {
       V1(RequestV1),
       V2(RequestV2),
   }
   ```

---

## 8. Migration Checklist

### Concurrency
- [ ] Replace goroutines with tokio::spawn
- [ ] Replace sync.Cond with tokio::sync::Notify
- [ ] Replace sync.Mutex with parking_lot::Mutex
- [ ] Implement rayon parallel matching
- [ ] Convert event loop to async/await

### Error Handling
- [ ] Add anyhow to application code
- [ ] Add thiserror to library modules
- [ ] Replace if err != nil with ? operator
- [ ] Add context to error chains
- [ ] Remove all unwrap() calls

### Data Structures
- [ ] Replace pointers with Arc
- [ ] Replace nil with Option
- [ ] Add proper lifetime annotations
- [ ] Use Vec instead of growable slices
- [ ] Use HashMap for maps

### Serialization
- [ ] Add serde derives
- [ ] Implement custom serializers where needed
- [ ] Test JSON compatibility with Go
- [ ] Add server API types

### Configuration
- [ ] Implement clap::Parser
- [ ] Add validation logic
- [ ] Implement tracing logging
- [ ] Add config file support (optional)
