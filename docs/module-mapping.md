# fzf Go→Rust Module Mapping

## Overview

| Metric | Value |
|--------|-------|
| **Total Go Lines** | 29,695 |
| **Total Rust Lines** | 5,564 |
| **Migration Progress** | 18.7% |
| **Modules Complete** | 0 / 18 |
| **Modules Partial** | 17 / 18 |
| **Modules Not Started** | 1 / 18 |

## Module Status Legend

| Status | Definition |
|--------|------------|
| **已完成** | Feature-complete Rust implementation matching Go functionality |
| **部分完成** | Rust implementation exists but incomplete vs Go |
| **未开始** | No Rust implementation exists |
| **已实现但待验证** | Rust code exists but needs testing/validation |

---

## Core Module Mapping

### 1. Core Runtime (`core`)

| Attribute | Details |
|-----------|---------|
| **Go Location** | `src/core.go` |
| **Go Lines** | 640 |
| **Rust Location** | `src/core.rs` |
| **Rust Lines** | 222 |
| **Line Ratio** | 34.7% |
| **Status** | **部分完成** |
| **Notes** | Event loop, Run() function stubbed; missing event coordination |

### 2. Options / CLI Parsing (`options`)

| Attribute | Details |
|-----------|---------|
| **Go Location** | `src/options.go` (+ `options_no_pprof.go`, `options_pprof.go`) |
| **Go Lines** | 4,039 |
| **Rust Location** | `src/options.rs` |
| **Rust Lines** | 678 |
| **Line Ratio** | 16.8% |
| **Status** | **部分完成** |
| **Notes** | Basic clap derive macros; missing many options from Go version |

### 3. Matcher Engine (`matcher`)

| Attribute | Details |
|-----------|---------|
| **Go Location** | `src/matcher.go` |
| **Go Lines** | 271 |
| **Rust Location** | `src/matcher.rs` |
| **Rust Lines** | 148 |
| **Line Ratio** | 54.6% |
| **Status** | **部分完成** |
| **Notes** | Parallel matching structure present; missing worker coordination |

### 4. Terminal UI (`terminal`)

| Attribute | Details |
|-----------|---------|
| **Go Location** | `src/terminal.go` (+ `terminal_unix.go`, `terminal_windows.go`) |
| **Go Lines** | 8,243 |
| **Rust Location** | `src/terminal.rs` |
| **Rust Lines** | 1,039 |
| **Line Ratio** | 12.6% |
| **Status** | **部分完成** |
| **Notes** | Large Go file; Rust has basic structure, ratatui integration started |

### 5. Pattern Parsing (`pattern`)

| Attribute | Details |
|-----------|---------|
| **Go Location** | `src/pattern.go` |
| **Go Lines** | 501 |
| **Rust Location** | `src/pattern.rs` |
| **Rust Lines** | 388 |
| **Line Ratio** | 77.4% |
| **Status** | **部分完成** |
| **Notes** | Query parsing mostly implemented; fuzzy/exact/prefix/suffix modes |

### 6. Item Model (`item`)

| Attribute | Details |
|-----------|---------|
| **Go Location** | `src/item.go` |
| **Go Lines** | 59 |
| **Rust Location** | `src/item.rs` |
| **Rust Lines** | 114 |
| **Line Ratio** | 193.2% |
| **Status** | **已实现但待验证** |
| **Notes** | Rust implementation has more type safety; needs behavior verification |

### 7. Results / Scoring (`result`)

| Attribute | Details |
|-----------|---------|
| **Go Location** | `src/result.go` (+ `result_others.go`, `result_x86.go`) |
| **Go Lines** | 461 |
| **Rust Location** | `src/result.rs` |
| **Rust Lines** | 113 |
| **Line Ratio** | 24.5% |
| **Status** | **部分完成** |
| **Notes** | Scoring structure present; missing platform-specific optimizations |

### 8. Merger (`merger`)

| Attribute | Details |
|-----------|---------|
| **Go Location** | `src/merger.go` |
| **Go Lines** | 183 |
| **Rust Location** | `src/merger.rs` |
| **Rust Lines** | 105 |
| **Line Ratio** | 57.4% |
| **Status** | **部分完成** |
| **Notes** | Result combining logic started |

### 9. Reader (`reader`)

| Attribute | Details |
|-----------|---------|
| **Go Location** | `src/reader.go` |
| **Go Lines** | 413 |
| **Rust Location** | `src/reader.rs` |
| **Rust Lines** | 141 |
| **Line Ratio** | 34.1% |
| **Status** | **部分完成** |
| **Notes** | Input handling stubbed; tokio async started |

### 10. Cache (`cache`)

| Attribute | Details |
|-----------|---------|
| **Go Location** | `src/cache.go` |
| **Go Lines** | 98 |
| **Rust Location** | `src/cache.rs` |
| **Rust Lines** | 81 |
| **Line Ratio** | 82.7% |
| **Status** | **部分完成** |
| **Notes** | Chunk-based caching structure present |

### 11. ANSI Processing (`ansi`)

| Attribute | Details |
|-----------|---------|
| **Go Location** | `src/ansi.go` |
| **Go Lines** | 578 |
| **Rust Location** | `src/ansi.rs` |
| **Rust Lines** | 69 |
| **Line Ratio** | 11.9% |
| **Status** | **部分完成** |
| **Notes** | Color code parsing stubbed |

### 12. Functions / Actions (`functions`)

| Attribute | Details |
|-----------|---------|
| **Go Location** | `src/functions.go` |
| **Go Lines** | 35 |
| **Rust Location** | `src/functions.rs` |
| **Rust Lines** | 445 |
| **Line Ratio** | 1271.4% |
| **Status** | **已实现但待验证** |
| **Notes** | Go delegates to other files; Rust has comprehensive action definitions |

### 13. Tokenizer (`tokenizer`)

| Attribute | Details |
|-----------|---------|
| **Go Location** | `src/tokenizer.go` |
| **Go Lines** | 338 |
| **Rust Location** | `src/tokenizer.rs` |
| **Rust Lines** | 73 |
| **Line Ratio** | 21.6% |
| **Status** | **部分完成** |
| **Notes** | Field extraction with delimiters stubbed |

### 14. History (`history`)

| Attribute | Details |
|-----------|---------|
| **Go Location** | `src/history.go` |
| **Go Lines** | 95 |
| **Rust Location** | `src/history.rs` |
| **Rust Lines** | 133 |
| **Line Ratio** | 140.0% |
| **Status** | **已实现但待验证** |
| **Notes** | Persistent history storage; needs verification |

### 15. Server (`server`)

| Attribute | Details |
|-----------|---------|
| **Go Location** | `src/server.go` |
| **Go Lines** | 276 |
| **Rust Location** | `src/server.rs` |
| **Rust Lines** | 40 |
| **Line Ratio** | 14.5% |
| **Status** | **部分完成** |
| **Notes** | HTTP server for remote control stubbed |

---

## Sub-package Module Mapping

### 16. Algorithms (`algo/`)

| Attribute | Details |
|-----------|---------|
| **Go Files** | `algo.go`, `normalize.go`, `indexbyte2_*.go`, `indexbyte2_*.s` |
| **Go Lines** | 2,622 (including assembly) |
| **Rust Files** | `mod.rs`, `v1.rs`, `v2.rs`, `normalize.rs`, `tests.rs` |
| **Rust Lines** | 1,021 |
| **Line Ratio** | 38.9% |
| **Status** | **部分完成** |
| **Notes** | V1/V2 algorithms started; missing SIMD assembly (amd64/arm64) |

**Detailed Breakdown:**

| File | Go Lines | Rust Lines | Status |
|------|----------|------------|--------|
| Core algo | 1,030 | 345 (mod.rs) | Partial |
| V1 algorithm | N/A (in algo.go) | 249 | In Progress |
| V2 algorithm | N/A (in algo.go) | 169 | In Progress |
| Normalization | 589 | 12 | Partial |
| SIMD (amd64) | Assembly + Go stub | N/A | **未开始** |
| SIMD (arm64) | Assembly + Go stub | N/A | **未开始** |

### 17. TUI (`tui/`)

| Attribute | Details |
|-----------|---------|
| **Go Files** | `tui.go`, `light.go`, `light_unix.go`, `light_windows.go`, `tcell.go`, etc. |
| **Go Lines** | 4,812 |
| **Rust Files** | `mod.rs`, `light.rs` |
| **Rust Lines** | 152 |
| **Line Ratio** | 3.2% |
| **Status** | **部分完成** |
| **Notes** | crossterm/ratatui backend started; missing light/tcell backends |

**Detailed Breakdown:**

| File | Go Lines | Rust Lines | Status |
|------|----------|------------|--------|
| TUI interface | 1,510 | 94 (mod.rs) | Partial |
| Light terminal | 1,561 + platform | 58 | Partial |
| tcell backend | 1,183 | N/A | **未开始** |
| Platform-specific | 558 (unix/windows) | N/A | **未开始** |

### 18. Utilities (`util/`)

| Attribute | Details |
|-----------|---------|
| **Go Files** | `util.go`, `chars.go`, `eventbox.go`, `slab.go`, `atexit.go`, `atomicbool.go`, `concurrent_set.go` |
| **Go Lines** | 1,023 |
| **Rust Files** | `mod.rs`, `chars.rs`, `eventbox.rs`, `slab.rs` |
| **Rust Lines** | 448 |
| **Line Ratio** | 43.8% |
| **Status** | **部分完成** |
| **Notes** | EventBox, slab allocator, chars utility present; missing atexit, atomicbool, concurrent_set |

**Detailed Breakdown:**

| File | Go Lines | Rust Lines | Status |
|------|----------|------------|--------|
| Utilities core | 163 | 80 (mod.rs) | Partial |
| Chars handling | 348 | 222 | Partial |
| EventBox | 96 | 100 | Partial |
| Slab allocator | 12 | 46 | In Progress |
| AtExit | 28 | N/A | **未开始** |
| AtomicBool | 34 | N/A | **未开始** |
| ConcurrentSet | 39 | N/A | **未开始** |

### 19. Protector (`protector/`)

| Attribute | Details |
|-----------|---------|
| **Go Files** | `protector.go`, `protector_openbsd.go` |
| **Go Lines** | 16 |
| **Rust Files** | None |
| **Rust Lines** | 0 |
| **Line Ratio** | 0% |
| **Status** | **未开始** |
| **Notes** | OpenBSD security pledge not applicable in Rust |

---

## Platform-Specific Code Mapping

### Unix-Specific

| Go File | Lines | Rust Equivalent | Status |
|---------|-------|-----------------|--------|
| `terminal_unix.go` | 24 | Partial in terminal.rs | Partial |
| `light_unix.go` | 181 | N/A | Not Started |
| `util_unix.go` | 94 | N/A | Not Started |
| `proxy_unix.go` | 41 | N/A | Not Started |
| `ttyname_unix.go` | 54 | N/A | Not Started |

### Windows-Specific

| Go File | Lines | Rust Equivalent | Status |
|---------|-------|-----------------|--------|
| `terminal_windows.go` | 15 | Partial in terminal.rs | Partial |
| `light_windows.go` | 183 | N/A | Not Started |
| `util_windows.go` | 226 | N/A | Not Started |
| `proxy_windows.go` | 85 | N/A | Not Started |
| `winpty.go` | 13 | N/A | Not Started |
| `winpty_windows.go` | 80 | N/A | Not Started |
| `ttyname_windows.go` | 21 | N/A | Not Started |

### Architecture-Specific (SIMD)

| Go File | Lines | Rust Equivalent | Status |
|---------|-------|-----------------|--------|
| `indexbyte2_amd64.go` | 24 + assembly | N/A | **未开始** |
| `indexbyte2_arm64.go` | 17 + assembly | N/A | **未开始** |
| `indexbyte2_other.go` | 33 | N/A | **未开始** |

---

## Entry Points

| Component | Go | Rust | Status |
|-----------|-----|------|--------|
| CLI Entry | `main.go` (104 lines) | `src/main.rs` (23 lines) | Partial |
| Library Root | N/A (implicit) | `src/lib.rs` (30 lines) | N/A |

---

## Summary Statistics

### By Status

| Status | Count | Modules |
|--------|-------|---------|
| **已完成** | 0 | - |
| **部分完成** | 15 | core, options, matcher, terminal, pattern, result, merger, reader, cache, ansi, functions, tokenizer, server, algo/, tui/, util/ |
| **已实现但待验证** | 3 | item, history, functions |
| **未开始** | 1 | protector/ (OpenBSD-specific) |

### By Line Coverage

| Coverage Range | Modules |
|----------------|---------|
| 0-20% | terminal (12.6%), ansi (11.9%), tui/ (3.2%) |
| 20-40% | options (16.8%), result (24.5%), tokenizer (21.6%), core (34.7%), reader (34.1%), algo/ (38.9%) |
| 40-60% | matcher (54.6%), merger (57.4%), util/ (43.8%) |
| 60-80% | pattern (77.4%), cache (82.7%) |
| 80-100%+ | item (193%), history (140%), functions (1271% - Go delegates) |

---

## Recommendations for Next Phase

### High Priority (Foundation)
1. **core** - Event loop coordination needed for all other modules
2. **matcher** - Central to fuzzy finding functionality
3. **reader** - Input pipeline foundation

### Medium Priority (Features)
4. **terminal** - Large but critical for UI
5. **options** - Many CLI features missing
6. **algo/** - SIMD optimizations can wait, but core algorithms needed

### Low Priority / Platform-Specific
7. **protector/** - OpenBSD only, may be skipped
8. **Windows-specific files** - Unix primary target
9. **SIMD assembly** - Performance optimization, not core functionality
