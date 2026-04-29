# fzf Rust Migration Project

## Project Overview

**Source Project**: fzf (command-line fuzzy finder)  
**Original Language**: Go (version 1.23+)  
**Target Language**: Rust  
**Lines of Code**: ~24,500 (Go)

## Original Project Structure

### Entry Points
- `main.go` - CLI entry point, version handling, shell integration output

### Core Modules (`src/`)

| Module | Purpose | Key Responsibilities |
|--------|---------|---------------------|
| `core.go` | Main runtime | Event loop coordination, Run() function |
| `options.go` | CLI parsing | Option definitions, parsing, validation |
| `matcher.go` | Search engine | Parallel matching, scoring, caching |
| `reader.go` | Input handling | Read from stdin, files, or command output |
| `terminal.go` | UI rendering | Terminal UI, key handling, display |
| `pattern.go` | Pattern parsing | Query parsing (fuzzy, exact, prefix, suffix) |
| `item.go` | Data model | Item representation, transformation |
| `result.go` | Match results | Scoring results, ranking |
| `merger.go` | Result merging | Combine results from parallel workers |
| `history.go` | Search history | Persistent history storage |
| `cache.go` | Match caching | Chunk-based result caching |
| `ansi.go` | ANSI processing | Color code parsing and handling |
| `functions.go` | Actions | User action handlers |
| `tokenizer.go` | Tokenization | Field extraction with delimiters |
| `server.go` | HTTP server | Remote control API |

### Sub-packages

| Package | Purpose |
|---------|---------|
| `src/algo/` | Fuzzy matching algorithms (V1, V2) with SIMD optimizations |
| `src/tui/` | Terminal abstraction (light/tcell backends) |
| `src/util/` | Utilities (chars, slab allocator, eventbox, atexit) |
| `src/protector/` | Security protection (OpenBSD pledge) |

### External Dependencies

```
- fastwalk      - Fast directory walking
- tcell/v2      - Terminal UI library
- go-shellwords - Shell-style string parsing
- go-isatty     - TTY detection
- uniseg        - Unicode segmentation
- golang.org/x/sys, golang.org/x/term - System/terminal utilities
```

### Build System
- Go Modules (`go.mod`)
- Makefile for build automation
- Supports cross-compilation for multiple platforms

### Testing
- Go unit tests (`*_test.go` files)
- Ruby integration tests (`test/` directory)

## Key Features to Migrate

1. **Fuzzy Matching**: Two algorithms (V1 - fast, V2 - optimal with scoring)
2. **Terminal UI**: Full-screen or height-limited display with preview window
3. **Shell Integration**: Key bindings and completions for bash/zsh/fish
4. **Search Modes**: Exact, fuzzy, prefix, suffix, inverse matching
5. **Scoring**: Word boundary bonuses, gap penalties, tiebreakers
6. **Actions**: Event-action binding system for customization
7. **Multi-select**: Tab/shift-tab selection with accept-nth
8. **Preview Window**: Live command preview with placeholder substitution
9. **Remote Control**: HTTP server mode for scripting
10. **Tmux/Zellij Integration**: Popup window support

## Architecture Notes

### Event-Driven Architecture
- EventBox for thread-safe event passing
- Reader -> Matcher -> Terminal pipeline
- Parallel matching with configurable workers

### Memory Management
- Custom slab allocator for small objects
- Chunk-based caching for search results
- Atomic operations for thread safety

### Platform Support
- Unix/Linux: Primary target
- Windows: Conditional compilation
- macOS: Supported
- OpenBSD: Security pledge support

## Rust Migration Strategy

### Recommended Crate Ecosystem

| Go Dependency | Rust Equivalent |
|--------------|-----------------|
| tcell/v2 | `crossterm` or `ratatui` |
| fastwalk | `walkdir` or `ignore` |
| go-shellwords | `shlex` |
| go-isatty | `atty` or `is-terminal` |
| uniseg | `unicode-segmentation` |
| golang.org/x/sys/term | `libc`, `termios` |

### Project Structure (Proposed)
```
fzf-rs/
├── Cargo.toml
├── src/
│   ├── main.rs           # Entry point
│   ├── lib.rs            # Library exports
│   ├── core.rs           # Main runtime
│   ├── options.rs        # CLI parsing
│   ├── matcher.rs        # Search engine
│   ├── reader.rs         # Input handling
│   ├── terminal.rs       # UI rendering
│   ├── pattern.rs        # Query parsing
│   ├── item.rs           # Data model
│   ├── result.rs         # Match results
│   ├── merger.rs         # Result merging
│   ├── history.rs        # Search history
│   ├── cache.rs          # Match caching
│   ├── ansi.rs           # ANSI processing
│   ├── functions.rs      # Actions
│   ├── tokenizer.rs      # Tokenization
│   ├── server.rs         # HTTP server
│   └── algo/             # Matching algorithms
│       ├── mod.rs
│       ├── v1.rs
│       ├── v2.rs
│       └── normalize.rs
│   ├── tui/              # Terminal abstraction
│   │   ├── mod.rs
│   │   ├── light.rs
│   │   └── event.rs
│   └── util/             # Utilities
│       ├── mod.rs
│       ├── chars.rs
│       ├── slab.rs
│       ├── eventbox.rs
│       └── atomic.rs
├── tests/                # Integration tests
├── shell/                # Shell integration scripts (reused)
└── man/                  # Man pages (reused)
```

## Acceptance Criteria Mapping

- **AC-1**: Document original structure (this document) ✓
- **AC-2**: Implement all core modules in Rust
- **AC-3**: CLI compatibility, same I/O behavior
- **AC-4**: Ubuntu build with Cargo
- **AC-5**: Unit tests + integration tests
- **AC-6**: BUILD.md with Rust instructions
- **AC-7**: Reuse Ruby test suite where possible
- **AC-8**: No new features, pure migration
