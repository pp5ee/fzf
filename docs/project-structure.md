# fzf Project Structure Analysis

## Project Overview

| Attribute | Value |
|-----------|-------|
| **Project Name** | fzf (command-line fuzzy finder) |
| **Original Language** | Go (version 1.23+) |
| **Target Language** | Rust (edition 2021) |
| **Project Type** | CLI application with library components |
| **Repository** | https://github.com/junegunn/fzf |
| **License** | MIT |

## Codebase Statistics

| Metric | Go | Rust | Ratio |
|--------|-----|------|-------|
| **Source Files** | 79 | 29 | 2.7:1 |
| **Lines of Code** | ~29,700 | ~5,600 | 5.3:1 |
| **Test Files** | 19 test files | 7 test modules | - |
| **Integration Tests** | ~7,000 lines Ruby | Partial | - |

## Directory Structure

```
/home/harness/cyops_data/workspace/rewrite-test/
├── main.go                    # Go entry point (CLI wrapper)
├── go.mod                     # Go module definition
├── Cargo.toml                 # Rust package definition
├── Cargo.lock                 # Rust dependency lock
├── Makefile                   # Build automation (Go-focused)
├── BUILD.md                   # Rust build instructions
├── CLAUDE.md                  # Migration project documentation
│
├── src/                       # Main source directory
│   ├── *.go                   # Go source files (44 root files)
│   ├── *.rs                   # Rust source files (17 root files)
│   │
│   ├── algo/                  # Fuzzy matching algorithms
│   │   ├── algo.go            # ~850 lines - Core algorithm interface
│   │   ├── algo_test.go       # Algorithm tests
│   │   ├── normalize.go       # ~700 lines - Unicode normalization
│   │   ├── indexbyte2_*.go    # SIMD optimizations (amd64, arm64)
│   │   ├── indexbyte2_*.s     # Assembly implementations
│   │   ├── mod.rs             # Rust algorithm module
│   │   ├── v1.rs              # ~200 lines - V1 algorithm
│   │   ├── v2.rs              # ~180 lines - V2 algorithm
│   │   ├── normalize.rs       # ~20 lines - Stub
│   │   └── tests.rs           # Rust tests
│   │
│   ├── tui/                   # Terminal UI abstraction
│   │   ├── tui.go             # ~1,100 lines - TUI interface
│   │   ├── light.go           # ~900 lines - Light terminal
│   │   ├── light_unix.go      # Unix-specific
│   │   ├── light_windows.go   # Windows-specific
│   │   ├── tcell.go           # ~700 lines - tcell backend
│   │   ├── mod.rs             # ~80 lines - Rust TUI module
│   │   └── light.rs           # ~50 lines - Light terminal stub
│   │
│   ├── util/                  # Utilities
│   │   ├── util.go            # ~110 lines
│   │   ├── chars.go           # ~240 lines - Character handling
│   │   ├── eventbox.go        # ~60 lines - Event passing
│   │   ├── slab.go            # ~60 lines - Slab allocator
│   │   ├── mod.rs             # ~60 lines
│   │   ├── chars.rs           # ~140 lines
│   │   ├── eventbox.rs        # ~60 lines
│   │   └── slab.rs            # ~40 lines
│   │
│   ├── protector/             # Security (OpenBSD pledge)
│   │   ├── protector.go       # ~10 lines
│   │   └── protector_openbsd.go
│   │
│   └── functions/             # Action handlers
│       └── tests.rs           # ~140 lines
│
├── shell/                     # Shell integration scripts
│   ├── key-bindings.bash      # Bash key bindings
│   ├── key-bindings.zsh       # Zsh key bindings
│   ├── key-bindings.fish      # Fish key bindings
│   ├── completion.bash        # Bash completion
│   ├── completion.zsh         # Zsh completion
│   └── completion.fish        # Fish completion
│
├── man/                       # Man pages
│   └── man1/
│       ├── fzf.1              # Main man page
│       └── fzf-tmux.1         # Tmux integration
│
├── test/                      # Ruby integration tests
│   ├── runner.rb              # Test runner
│   ├── rust_verify.rb         # Simplified Rust verification
│   ├── test_core.rb           # ~2,500 lines - Core tests
│   ├── test_layout.rb         # ~1,700 lines - Layout tests
│   ├── test_exec.rb           # ~500 lines - Execution tests
│   ├── test_filter.rb         # ~350 lines - Filter tests
│   ├── test_preview.rb        # ~850 lines - Preview tests
│   ├── test_shell_integration.rb  # ~1,200 lines
│   ├── test_server.rb         # ~100 lines
│   └── test_raw.rb            # ~100 lines
│
├── tests/                     # Additional tests (currently empty)
├── benches/                   # Benchmarks
├── bin/                       # Built binaries
├── plugin/                    # Vim plugin
├── doc/                       # Documentation
└── target/                    # Rust build output
    ├── debug/
    └── release/
```

## Build System

### Go Build
- **Entry Point**: `main.go`
- **Build Tool**: `make` (Makefile)
- **Module**: `github.com/junegunn/fzf`
- **Dependencies**: 11 direct + indirect
  - fastwalk (directory walking)
  - tcell/v2 (terminal UI)
  - go-shellwords (shell parsing)
  - go-isatty (TTY detection)
  - uniseg (Unicode segmentation)
  - golang.org/x/sys, golang.org/x/term

### Rust Build
- **Entry Point**: `src/main.rs`
- **Build Tool**: `cargo`
- **Package**: `fzf` v0.72.0
- **Dependencies**: 22 crates
  - clap (CLI parsing)
  - crossterm (terminal)
  - ratatui (TUI library)
  - tokio (async runtime)
  - regex, rayon, serde, tracing

## Module Mapping Summary

| Module | Go Files | Rust Files | Status |
|--------|----------|------------|--------|
| Core runtime | core.go (~650 lines) | core.rs (~150 lines) | Partial |
| Options/CLI | options.go (~2,800 lines) | options.rs (~400 lines) | Partial |
| Matcher | matcher.go (~750 lines) | matcher.rs (~200 lines) | Partial |
| Terminal UI | terminal.go (~1,500 lines) | terminal.rs (~350 lines) | Partial |
| Pattern parsing | pattern.go (~550 lines) | pattern.rs (~200 lines) | Partial |
| Item handling | item.go (~350 lines) | item.rs (~150 lines) | Partial |
| Results/Scoring | result.go (~400 lines) | result.rs (~100 lines) | Partial |
| Merger | merger.go (~150 lines) | merger.rs (~80 lines) | Partial |
| Reader | reader.go (~450 lines) | reader.rs (~200 lines) | Partial |
| Cache | cache.go (~200 lines) | cache.rs (~50 lines) | Partial |
| ANSI processing | ansi.go (~350 lines) | ansi.rs (~150 lines) | Partial |
| Functions/Actions | functions.go (~1,100 lines) | functions.rs (~300 lines) | Partial |
| Tokenizer | tokenizer.go (~150 lines) | tokenizer.rs (~80 lines) | Partial |
| History | history.go (~250 lines) | history.rs (~100 lines) | Partial |
| Server | server.go (~350 lines) | server.rs (~100 lines) | Partial |
| Algorithms | algo/ (2,800+ lines) | algo/ (500+ lines) | Partial |
| TUI | tui/ (6,500+ lines) | tui/ (200+ lines) | Partial |
| Utilities | util/ (1,200+ lines) | util/ (400+ lines) | Partial |
| Protector | protector/ (2 files) | None | Not started |

## Test Infrastructure

| Test Type | Go | Rust | Status |
|-----------|-----|------|--------|
| Unit tests | 19 _test.go files | 7 inline test modules | Partial |
| Integration tests | Ruby suite (~7,000 lines) | rust_verify.rb (basic) | Partial |
| CI/CD | None in .github/workflows | None configured | Missing |

## Key Observations

1. **Migration Progress**: ~19% of Go code migrated to Rust (by line count)
2. **Architecture**: Both Go and Rust use modular structure with similar module names
3. **Platform Support**: Go has Unix/Windows variants; Rust has basic cross-platform
4. **Missing in Rust**:
   - SIMD optimizations (assembly files)
   - OpenBSD security pledge
   - Platform-specific terminal handling
   - Complete algorithm implementations
5. **Shell Integration**: Reused across both implementations (shell scripts unchanged)
6. **Documentation**: Man pages and READMEs language-agnostic

## Language Distribution

| Directory | Go Files | Rust Files | Notes |
|-----------|----------|------------|-------|
| src/ root | 44 | 17 | Core modules |
| src/algo/ | 6 | 5 | Matching algorithms |
| src/tui/ | 9 | 2 | Terminal abstraction |
| src/util/ | 9 | 5 | Utilities |
| src/protector/ | 2 | 0 | Security (Go only) |
| src/functions/ | 0 | 1 | Action handlers (tests only) |
| **Total** | **70** | **30** | (excludes tests) |
