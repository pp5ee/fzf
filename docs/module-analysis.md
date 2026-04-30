# fzf Go→Rust Module Analysis: Dependencies, Interfaces & Risks

## Overview

This document provides detailed analysis of all incomplete Rust modules, including:
- Go source locations and Rust corresponding locations
- Dependency graphs
- External interfaces
- Risk assessments
- Recommended priorities

---

## Priority 1: Core Foundation (Critical Path)

### 1. Core Runtime (`core`)

| Attribute | Details |
|-----------|---------|
| **Go Source** | `src/core.go` (640 lines) |
| **Rust Source** | `src/core.rs` (223 lines) |
| **Migration %** | 34.7% |
| **Status** | 部分完成 |

#### Dependencies
```
Go imports:
- "github.com/junegunn/fzf/src/tui"
- "github.com/junegunn/fzf/src/util"

Rust imports:
- crate::cache::ChunkCache
- crate::matcher::Matcher
- crate::options::{Options, Scheme}
- crate::pattern::{Pattern, PatternOptions}
- crate::reader::{Reader, EVT_READ_FIN, EVT_READ_NEW}
- crate::terminal::Terminal
- crate::util::eventbox::{EventBox, EventType, EventValue}
- crate::util::Executor
- std::sync::atomic::{AtomicBool, Ordering}
- anyhow::{Result, anyhow}
```

#### External Interfaces

| Interface | Go | Rust | Gap |
|-----------|-----|------|-----|
| Event loop coordination | Full | Filter mode only | Missing interactive mode loop |
| Reader→Matcher pipeline | EventBox channels | Direct calls | No async event passing |
| Terminal integration | Full | Stub | `Terminal::run()` not implemented |
| Tmux/Zellij support | `runTmux()`, `runZellij()` | Missing | Popup window support absent |
| Windows support | `runWinpty()`, `needWinpty()` | Missing | Windows terminal handling absent |
| Post-process options | `postProcessOptions()` | Missing | Option validation incomplete |

#### Risk Assessment

| Risk | Level | Impact | Mitigation |
|------|-------|--------|------------|
| Missing event-driven architecture | **HIGH** | Core functionality blocked | Implement EventBox-based async coordination |
| Incomplete terminal integration | **HIGH** | Interactive mode unusable | Connect Terminal to event loop |
| Missing tmux/zellij support | MEDIUM | Feature parity gap | Can be deferred; basic fzf works without |
| ANSI processing incomplete | MEDIUM | Color handling issues | Partial implementation exists |

#### Implementation Priority: **CRITICAL (P0)**

---

### 2. Options / CLI Parsing (`options`)

| Attribute | Details |
|-----------|---------|
| **Go Source** | `src/options.go` (3,953 lines) + helpers |
| **Rust Source** | `src/options.rs` (678+ lines) |
| **Migration %** | 16.8% |
| **Status** | 部分完成 |

#### Dependencies
```
Go imports:
- "github.com/junegunn/fzf/src/algo"
- "github.com/junegunn/fzf/src/tui"
- "github.com/junegunn/fzf/src/util"
- "github.com/junegunn/go-shellwords"
- "github.com/rivo/uniseg"

Rust imports:
- clap::{Arg, ArgAction, Command}
- Basic types defined locally
```

#### External Interfaces

| Interface | Go | Rust | Gap |
|-----------|-----|------|-----|
| Command-line parsing | 150+ options | ~30 options | Most options missing |
| Shell command parsing | `go-shellwords` | `shlex` crate | Syntax differences possible |
| Unicode segmentation | `uniseg` | `unicode-segmentation` | API differences |
| Validation logic | Extensive | Minimal | Invalid inputs may pass |
| Action bindings | `parseKeyChords()` | Missing | Key binding parsing absent |
| Color scheme parsing | `parseColor()` | Missing | --color option broken |
| Preview parsing | `parsePreviewWindow()` | Missing | Preview window config absent |
| History management | Full | Partial | History options incomplete |

#### Missing Go Options (Critical)

```go
// Layout and display
--style, --gap, --gap-line, --freeze-left, --freeze-right
--highlight-line, --wrap, --wrap-sign, --no-multi-line
--raw, --track, --id-nth

// Preview
--preview-window detailed options, --preview-border
--preview-label, --preview-label-pos, --preview-wrap-sign

// Header/Footer borders
--header-border, --header-lines-border, --header-label
--footer, --footer-border, --footer-label

// Input section
--no-input, --separator, --no-separator, --ghost
--filepath-word, --input-border, --input-label

// List section
--list-border, --list-label, --scrollbar, --no-scrollbar
--pointer variants, --marker variants, --gutter variants

// Advanced
--tmux, --popup (detailed options)
--listen (server mode config)
--bind actions (complex action syntax)
--expect keys
```

#### Risk Assessment

| Risk | Level | Impact | Mitigation |
|------|-------|--------|------------|
| Missing CLI options | **HIGH** | User scripts broken | Incremental option addition |
| Shell command parsing mismatch | MEDIUM | Command execution differences | Test with edge cases |
| Action binding parser absent | **HIGH** | Custom key bindings fail | Priority implementation |
| Preview window unconfigurable | MEDIUM | Preview feature broken | Add basic preview support |

#### Implementation Priority: **CRITICAL (P0)**

---

### 3. Matcher Engine (`matcher`)

| Attribute | Details |
|-----------|---------|
| **Go Source** | `src/matcher.go` (271 lines) |
| **Rust Source** | `src/matcher.rs` (149 lines) |
| **Migration %** | 54.6% |
| **Status** | 部分完成 |

#### Dependencies
```
Go imports:
- "github.com/junegunn/fzf/src/util"
- "runtime", "sync", "sync/atomic"

Rust imports:
- crate::algo::{Algo, fuzzy_match_v1, fuzzy_match_v2}
- crate::cache::ChunkCache
- crate::item::Item
- crate::merger::Merger
- crate::pattern::Pattern
- crate::result::{Rank, Result}
- crate::util::chars::Chunk
- crate::util::eventbox
- std::sync::atomic::{AtomicBool, Ordering}
```

#### External Interfaces

| Interface | Go | Rust | Gap |
|-----------|-----|------|-----|
| Parallel matching | `partitions` workers | Single-threaded | Missing parallelism |
| Worker coordination | `Loop()` with reqBox | `loop_matcher()` stub | No worker pool |
| Slab allocator per worker | `[]*util.Slab` | Missing | Memory optimization absent |
| Sort buffer per worker | `[][]Result` | Missing | Parallel sort missing |
| Merger cache | `map[string]MatchResult` | Missing | Result caching absent |
| Cancellation | `AtomicBool` | `AtomicBool` | ✅ Implemented |
| Result ranking | Full | Partial | Ranking criteria missing |

#### Risk Assessment

| Risk | Level | Impact | Mitigation |
|------|-------|--------|------------|
| Single-threaded matching | **HIGH** | Performance degradation 10-100x | Implement rayon-based parallelism |
| Missing slab allocator | MEDIUM | Memory pressure | Use arena allocation |
| No merger caching | MEDIUM | Repeated searches slow | Add LRU cache |
| Pattern builder missing | MEDIUM | Dynamic pattern updates | Implement pattern builder |

#### Implementation Priority: **HIGH (P1)**

---

### 4. Reader (`reader`)

| Attribute | Details |
|-----------|---------|
| **Go Source** | `src/reader.go` (413 lines) |
| **Rust Source** | `src/reader.rs` (142 lines) |
| **Migration %** | 34.1% |
| **Status** | 部分完成 |

#### Dependencies
```
Go imports:
- "github.com/charlievieth/fastwalk"
- "github.com/junegunn/fzf/src/util"

Rust imports:
- crate::util::chars::{Chars, Chunk, ChunkList}
- crate::util::eventbox
- crate::util::Executor
- std::io::{self, BufRead, Read}
```

#### External Interfaces

| Interface | Go | Rust | Gap |
|-----------|-----|------|-----|
| Stdin reading | Full | `read_stdin()` | ✅ Basic reading works |
| Command execution | `readCommand()` | `read_command()` | Partial, async missing |
| File walking | `fastwalk` | Missing | Directory traversal absent |
| Event polling | `startEventPoller()` | Missing | No event notification |
| Termination | `terminate()` with callback | Missing | Graceful shutdown absent |
| Streaming | `streamingFilter` optimization | Missing | Streaming mode absent |
| Delimiter handling | `
` or `\0` | ✅ | Both supported |

#### Risk Assessment

| Risk | Level | Impact | Mitigation |
|------|-------|--------|------------|
| No file walker | **HIGH** | Cannot use fzf with file paths | Port `walkdir` usage |
| Missing event polling | **HIGH** | Terminal not notified of new items | Implement async events |
| No streaming optimization | MEDIUM | Filter mode slower | Add streaming path |
| Command async incomplete | MEDIUM | Reload feature broken | Complete tokio integration |

#### Implementation Priority: **HIGH (P1)**

---

## Priority 2: Terminal & UI (User-Facing)

### 5. Terminal (`terminal`)

| Attribute | Details |
|-----------|---------|
| **Go Source** | `src/terminal.go` (8,243 lines) + platform files |
| **Rust Source** | `src/terminal.rs` (1,039 lines) |
| **Migration %** | 12.6% |
| **Status** | 部分完成 |

#### Dependencies
```
Go imports:
- "github.com/rivo/uniseg"
- "github.com/junegunn/fzf/src/tui"
- "github.com/junegunn/fzf/src/util"
- Standard: context, encoding/json, net, os/exec, regexp, sync

Rust imports:
- crossterm, ratatui
- tokio (async)
- Basic types from other modules
```

#### External Interfaces

| Interface | Go | Rust | Gap |
|-----------|-----|------|-----|
| Terminal I/O | tcell + light | crossterm + ratatui | Architecture migrated |
| Window management | Full | Layout structure | Layout calculation missing |
| Preview window | Extensive | Missing | Preview completely absent |
| Placeholder expansion | `replacePlaceholder()` | Missing | Command templates broken |
| Key event handling | Full event loop | Stub | Key handling incomplete |
| Mouse support | Full | Missing | --no-mouse option irrelevant |
| Tmux integration | `runTmux()` | Missing | Popup support absent |
| Zellij integration | `runZellij()` | Missing | Popup support absent |
| Server mode | HTTP server | Missing | --listen broken |

#### Risk Assessment

| Risk | Level | Impact | Mitigation |
|------|-------|--------|------------|
| Preview window missing | **HIGH** | Major feature unavailable | Priority implementation |
| Placeholder expansion absent | **HIGH** | Shell integration broken | Implement template parser |
| Key bindings incomplete | **HIGH** | User interaction limited | Complete event handling |
| Layout engine incomplete | MEDIUM | UI rendering issues | Port layout algorithm |
| Server mode missing | MEDIUM | Remote control broken | Add HTTP server |

#### Implementation Priority: **HIGH (P1)**

---

### 6. TUI Sub-package (`tui/`)

| Attribute | Details |
|-----------|---------|
| **Go Source** | `tui/*.go` (4,812 lines) |
| **Rust Source** | `tui/mod.rs`, `tui/light.rs` (152 lines) |
| **Migration %** | 3.2% |
| **Status** | 部分完成 |

#### Dependencies
```
Go imports:
- "github.com/gdamore/tcell/v2"
- "github.com/junegunn/fzf/src/util"

Rust imports:
- crossterm, ratatui
```

#### External Interfaces

| Component | Go | Rust | Gap |
|-----------|-----|------|-----|
| tcell backend | Full tcell integration | N/A | Replaced by ratatui |
| Light terminal | Full implementation | Stub | Basic ANSI handling |
| Unix-specific | `light_unix.go` (181 lines) | N/A | PTY handling absent |
| Windows-specific | `light_windows.go` (183 lines) | N/A | Windows console absent |
| Event types | `eventtype_string.go` | Enum definitions | Event mapping incomplete |

#### Risk Assessment

| Risk | Level | Impact | Mitigation |
|------|-------|--------|------------|
| tcell→ratatui migration | MEDIUM | API differences | Abstract interface layer |
| Platform-specific terminals | MEDIUM | Unix/Windows features | Port platform code |
| Terminal capability detection | MEDIUM | Feature detection missing | Implement terminfo |

#### Implementation Priority: **MEDIUM (P2)**

---

## Priority 3: Algorithms & Matching

### 7. Algorithms (`algo/`)

| Attribute | Details |
|-----------|---------|
| **Go Source** | `algo/*.go` + assembly (2,622 lines) |
| **Rust Source** | `algo/*.rs` (1,021 lines) |
| **Migration %** | 38.9% |
| **Status** | 部分完成 |

#### Dependencies
```
Go imports:
- "github.com/junegunn/fzf/src/util"
- Architecture-specific assembly

Rust imports:
- Internal module organization
- Standard library only
```

#### External Interfaces

| Algorithm | Go | Rust | Gap |
|-----------|-----|------|-----|
| Fuzzy V1 | `algo.go` | `v1.rs` | ✅ Core algorithm present |
| Fuzzy V2 | `algo.go` | `v2.rs` | ✅ Core algorithm present |
| SIMD (amd64) | Assembly + Go | Missing | **未开始** |
| SIMD (arm64) | Assembly + Go | Missing | **未开始** |
| Normalization | `normalize.go` (589 lines) | `normalize.rs` (12 lines) | Partial |
| Bonus calculation | Full | Partial | Scoring differences likely |

#### Risk Assessment

| Risk | Level | Impact | Mitigation |
|------|-------|--------|------------|
| Missing SIMD | MEDIUM | Performance 2-5x slower | Port or use Rust SIMD |
| Normalization incomplete | MEDIUM | Unicode handling issues | Port full normalization |
| Scoring mismatch | **HIGH** | Different ranking results | Extensive testing needed |

#### Implementation Priority: **MEDIUM (P2)**

---

### 8. Pattern (`pattern`)

| Attribute | Details |
|-----------|---------|
| **Go Source** | `src/pattern.go` (501 lines) |
| **Rust Source** | `src/pattern.rs` (388 lines) |
| **Migration %** | 77.4% |
| **Status** | 部分完成 |

#### Dependencies
```
Go imports:
- "github.com/junegunn/fzf/src/algo"
- "github.com/junegunn/fzf/src/util"

Rust imports:
- crate::algo
- regex crate
```

#### External Interfaces

| Feature | Go | Rust | Gap |
|---------|-----|------|-----|
| Fuzzy matching | ✅ | ✅ | Implemented |
| Exact matching | ✅ | ✅ | Implemented |
| Prefix/suffix | ✅ | ✅ | Implemented |
| Inverse matching | ✅ | Partial | May have edge cases |
| Extended search | ✅ | Partial | OR/AND syntax incomplete |
| Case handling | ✅ | ✅ | Implemented |
| Normalization | ✅ | Partial | Unicode normalization gap |

#### Risk Assessment

| Risk | Level | Impact | Mitigation |
|------|-------|--------|------------|
| Extended search incomplete | MEDIUM | Advanced queries fail | Complete parser |
| Normalization gap | MEDIUM | Case folding differences | Use unicode-normalization |

#### Implementation Priority: **LOW (P3)**

---

## Priority 4: Supporting Modules

### 9. Results & Merger (`result`, `merger`)

| Attribute | Details |
|-----------|---------|
| **Go Result** | `src/result.go` (461 lines) |
| **Rust Result** | `src/result.rs` (113 lines) |
| **Go Merger** | `src/merger.go` (183 lines) |
| **Rust Merger** | `src/merger.rs` (105 lines) |
| **Migration %** | 24.5% / 57.4% |
| **Status** | 部分完成 |

#### External Interfaces

| Feature | Go | Rust | Gap |
|---------|-----|------|-----|
| Result ranking | Full criteria | Basic | Tiebreakers missing |
| Platform optimization | `result_x86.go` | N/A | **未开始** |
| Merger caching | Full | Missing | Performance impact |
| Pass-through merger | `passMerger` | Missing | Pass-through mode broken |

#### Implementation Priority: **MEDIUM (P2)**

---

### 10. Cache (`cache`)

| Attribute | Details |
|-----------|---------|
| **Go Source** | `src/cache.go` (98 lines) |
| **Rust Source** | `src/cache.rs` (81 lines) |
| **Migration %** | 82.7% |
| **Status** | 部分完成 |

#### Risk Assessment

| Risk | Level | Impact | Mitigation |
|------|-------|--------|------------|
| Chunk cache implementation | LOW | Structure present | Minor gaps only |

#### Implementation Priority: **LOW (P3)**

---

### 11. History (`history`)

| Attribute | Details |
|-----------|---------|
| **Go Source** | `src/history.go` (95 lines) |
| **Rust Source** | `src/history.rs` (133 lines) |
| **Migration %** | 140% |
| **Status** | 已实现但待验证 |

#### Risk Assessment

| Risk | Level | Impact | Mitigation |
|------|-------|--------|------------|
| Persistence verification | MEDIUM | History may not save/load | Test file I/O |
| Circular buffer logic | LOW | May differ from Go | Verify edge cases |

#### Implementation Priority: **LOW (P3)**

---

### 12. Server (`server`)

| Attribute | Details |
|-----------|---------|
| **Go Source** | `src/server.go` (276 lines) |
| **Rust Source** | `src/server.rs` (40 lines) |
| **Migration %** | 14.5% |
| **Status** | 部分完成 |

#### External Interfaces

| Feature | Go | Rust | Gap |
|---------|-----|------|-----|
| HTTP server | Full | Stub | --listen option broken |
| JSON API | Full | Missing | Remote control absent |
| WebSocket | Considered | N/A | Not in scope |

#### Implementation Priority: **LOW (P3)**

---

### 13. ANSI Processing (`ansi`)

| Attribute | Details |
|-----------|---------|
| **Go Source** | `src/ansi.go` (578 lines) |
| **Rust Source** | `src/ansi.rs` (69 lines) |
| **Migration %** | 11.9% |
| **Status** | 部分完成 |

#### Risk Assessment

| Risk | Level | Impact | Mitigation |
|------|-------|--------|------------|
| Color code parsing | **HIGH** | --ansi option broken | Port full parser |
| 256/truecolor support | Missing | Missing | Feature parity |
| Sixel/kitty graphics | Missing | Missing | Advanced features |

#### Implementation Priority: **MEDIUM (P2)**

---

### 14. Tokenizer (`tokenizer`)

| Attribute | Details |
|-----------|---------|
| **Go Source** | `src/tokenizer.go` (338 lines) |
| **Rust Source** | `src/tokenizer.rs` (73 lines) |
| **Migration %** | 21.6% |
| **Status** | 部分完成 |

#### Risk Assessment

| Risk | Level | Impact | Mitigation |
|------|-------|--------|------------|
| Field extraction | MEDIUM | --nth option broken | Complete tokenizer |
| Delimiter regex | Missing | AWK-style parsing | Implement regex delimiters |

#### Implementation Priority: **MEDIUM (P2)**

---

### 15. Functions/Actions (`functions`)

| Attribute | Details |
|-----------|---------|
| **Go Source** | `src/functions.go` (35 lines, delegates) |
| **Rust Source** | `src/functions.rs` (445 lines) |
| **Migration %** | 1271% (Go delegates) |
| **Status** | 已实现但待验证 |

#### Risk Assessment

| Risk | Level | Impact | Mitigation |
|------|-------|--------|------------|
| Action binding | MEDIUM | Key actions may differ | Comprehensive testing |
| Action types | Extensive | Partial | Many actions missing |

#### Implementation Priority: **HIGH (P1)**

---

## Priority 5: Platform-Specific (Deferrable)

### 16. Platform-Specific Code

| Platform | Go Files | Rust Status | Risk |
|----------|----------|-------------|------|
| Unix terminal | `terminal_unix.go` (24 lines) | Partial | LOW |
| Windows terminal | `terminal_windows.go` (15 lines) | Missing | LOW |
| Unix light | `light_unix.go` (181 lines) | Missing | MEDIUM |
| Windows light | `light_windows.go` (183 lines) | Missing | MEDIUM |
| Unix utils | `util_unix.go` (94 lines) | Missing | LOW |
| Windows utils | `util_windows.go` (226 lines) | Missing | LOW |
| Winpty | `winpty*.go` (93 lines) | **N/A** | Windows legacy |

### 17. Protector (`protector/`)

| Attribute | Details |
|-----------|---------|
| **Go Source** | `protector_openbsd.go` (10 lines) |
| **Rust Source** | None |
| **Status** | 未开始 |

**Note**: OpenBSD pledge security feature. Can be skipped for MVP.

#### Implementation Priority: **LOWEST (P4)**

---

## Dependency Graph

```
                    [core]
                   /  |   \
                  /   |    \
            [reader] [matcher] [terminal]
               |    /    |    /    |
               |   /     |   /     |
            [pattern] [merger]  [tui]
               |    \    |    /    |
               |     \   |   /     |
            [algo]   [result]    [util]
               |        |         |
               +--------+---------+
                        |
                    [options]
```

### Critical Path Dependencies

1. **options** → core, terminal, matcher, reader
2. **core** → reader, matcher, terminal, pattern
3. **matcher** → pattern, algo, result, merger
4. **reader** → util
5. **terminal** → tui, util
6. **tui** → (crossterm/ratatui external)

---

## Risk Summary by Priority

### P0 - Critical (Blockers)

| Module | Risk | Mitigation Effort |
|--------|------|-------------------|
| core | Event loop incomplete | 2-3 weeks |
| options | CLI parsing incomplete | 1-2 weeks |

### P1 - High (Major Features)

| Module | Risk | Mitigation Effort |
|--------|------|-------------------|
| matcher | Single-threaded | 1 week |
| reader | No file walker | 3-5 days |
| terminal | Preview missing | 2-3 weeks |
| functions | Actions incomplete | 1 week |

### P2 - Medium (Features & Polish)

| Module | Risk | Mitigation Effort |
|--------|------|-------------------|
| tui | Platform support | 1 week |
| algo | SIMD missing | 1 week |
| result/merger | Ranking/caching | 3-5 days |
| ansi | Color parsing | 1 week |
| tokenizer | Field extraction | 3-5 days |

### P3 - Low (Nice to Have)

| Module | Risk | Mitigation Effort |
|--------|------|-------------------|
| pattern | Extended search | 3-5 days |
| cache | Minor gaps | 1-2 days |
| history | Verification | 2-3 days |
| server | HTTP API | 1 week |

### P4 - Lowest (Deferrable)

| Module | Risk | Mitigation Effort |
|--------|------|-------------------|
| protector | OpenBSD only | Skip for MVP |
| Platform code | Unix primary | Add later |

---

## Recommended Implementation Order

```
Phase 1: Foundation (Weeks 1-3)
├── core - Event loop completion
├── options - Critical CLI options
└── reader - File walker + async

Phase 2: Matching (Weeks 4-5)
├── matcher - Parallelism
├── algo - SIMD (optional)
└── pattern - Extended search

Phase 3: Terminal (Weeks 6-8)
├── tui - Platform support
├── terminal - Preview window
└── ansi - Color processing

Phase 4: Polish (Weeks 9-10)
├── functions - Action bindings
├── result/merger - Caching
├── tokenizer - Field extraction
└── server - HTTP API

Phase 5: Verification (Week 11+)
├── history - Verification
├── Platform-specific code
└── Integration testing
```

---

## Go→Rust External Dependency Mapping

| Go Package | Rust Crate | Purpose | Risk |
|------------|------------|---------|------|
| `tcell/v2` | `crossterm` + `ratatui` | Terminal UI | MEDIUM (API differences) |
| `fastwalk` | `walkdir` | Directory walking | LOW |
| `go-shellwords` | `shlex` | Shell parsing | LOW |
| `go-isatty` | `atty`/`is-terminal` | TTY detection | LOW |
| `uniseg` | `unicode-segmentation` | Unicode | LOW |
| `golang.org/x/sys` | `libc`, `nix` | System calls | MEDIUM |
| `golang.org/x/term` | `crossterm` | Terminal | LOW |
| N/A (SIMD) | `packed_simd` | SIMD | MEDIUM |

---

## Interface Compatibility Notes

### Event System
- Go: `EventBox` with type+value pairs
- Rust: Similar structure but async handling differs
- Risk: Event ordering may differ

### Error Handling
- Go: `error` interface, multiple returns
- Rust: `Result<T, E>`, `anyhow`/`thiserror`
- Risk: Error messages may differ

### Concurrency
- Go: Goroutines + channels
- Rust: `tokio` async + `crossbeam` channels
- Risk: Deadlocks if ordering not preserved

### Memory Management
- Go: GC, slab allocator for performance
- Rust: Ownership + arena allocation
- Risk: Lifetime issues with cross-references
