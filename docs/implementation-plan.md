# fzf Go→Rust Migration: Phased Implementation Plan

## Executive Summary

| Item | Details |
|------|---------|
| **Project** | fzf (command-line fuzzy finder) |
| **Migration** | Go → Rust |
| **Current Progress** | 18.7% (5,564 / 29,695 lines) |
| **Estimated Duration** | 12-16 weeks |
| **Team Size** | 2-3 engineers |
| **Target** | Feature parity with Go v0.72.0 |

---

## Phase Overview

| Phase | Name | Duration | Focus | Deliverables |
|-------|------|----------|-------|--------------|
| **Phase 0** | 现状扫描 (Current State) | 1 week | Validate AC-1/2/3, establish baselines | Baseline report, build verification |
| **Phase 1** | 差异确认与架构对齐 (Gap Analysis) | 2 weeks | Architecture decisions, interface design | Architecture doc, interface specs |
| **Phase 2** | 核心转写补齐 (Core Implementation) | 6-8 weeks | P0/P1 modules, critical path | Working CLI, core features |
| **Phase 3** | 验证与收尾 (Verification) | 3-4 weeks | Testing, CI/CD, documentation | Production-ready release |

---

## Phase 0: 现状扫描 (Current State Analysis)

**Duration**: 1 week  
**Goal**: Validate findings from AC-1/2/3 and establish measurable baselines

### Week 0 Tasks

| Day | Task | Owner | Deliverable |
|-----|------|-------|-------------|
| 1-2 | Build verification | Dev | Build succeeds on Ubuntu |
| 2-3 | Test baseline | QA | Current test pass rate |
| 3-4 | Performance baseline | Perf | Benchmark numbers |
| 4-5 | Documentation review | Tech Lead | Gap report |

### Deliverables

```markdown
1. build-baseline.md
   - Go build: `make` succeeds
   - Rust build: `cargo build` succeeds
   - Platform: Ubuntu 22.04 LTS

2. test-baseline.md
   - Go unit tests: `make test` results
   - Rust unit tests: `cargo test` results
   - Ruby integration: `ruby test/runner.rb` results

3. performance-baseline.md
   - Filter mode: 1M items throughput
   - Memory usage: baseline RSS
   - Startup time: cold/warm

4. gap-validation.md
   - Confirms AC-2/AC-3 findings
   - Identifies any discrepancies
```

### Success Criteria
- [ ] Go binary builds and passes all tests
- [ ] Rust binary builds (may have limited functionality)
- [ ] Baseline metrics recorded
- [ ] Gap validation report complete

---

## Phase 1: 差异确认与架构对齐 (Gap Analysis & Architecture)

**Duration**: 2 weeks  
**Goal**: Finalize architecture decisions and interface contracts

### Week 1: Architecture Design

| Module | Decision Needed | Options | Recommendation |
|--------|-----------------|---------|----------------|
| **Event System** | Async runtime | tokio vs async-std | tokio (already in Cargo.toml) |
| **Parallelism** | Worker pool | rayon vs tokio tasks | rayon for CPU-bound matching |
| **Terminal** | Backend | crossterm vs termion | crossterm + ratatui |
| **SIMD** | Assembly port | inline asm vs packed_simd | Start with scalar, add SIMD later |
| **Error Handling** | Strategy | anyhow vs thiserror | anyhow for app, thiserror for libs |

### Week 2: Interface Specifications

```rust
// Core Event Loop Interface (src/core.rs)
pub trait EventLoop {
    fn run(&mut self) -> Result<i32>;
    fn handle_event(&mut self, event: Event) -> Result<()>;
}

// Matcher Interface (src/matcher.rs)
pub trait MatcherEngine {
    fn match_parallel(&self, items: &[Item], pattern: &Pattern) -> Vec<Result>;
    fn cancel(&self);
}

// Terminal Interface (src/terminal.rs)
pub trait TerminalDriver {
    fn run_interactive(&mut self, state: AppState) -> Result<Selection>;
    fn render(&mut self, items: &[Item], cursor: usize);
}
```

### Deliverables

```markdown
1. architecture-decisions.md (ADRs)
   - ADR-001: Async Runtime Selection
   - ADR-002: Concurrency Model
   - ADR-003: Terminal Abstraction
   - ADR-004: Memory Management Strategy
   - ADR-005: Error Handling Approach

2. interface-specs.md
   - Public API definitions
   - Trait boundaries
   - Module contracts

3. go-rust-mapping.md (Updated)
   - Detailed type mappings
   - Function signatures
   - Behavior specifications

4. testing-strategy.md
   - Unit test approach
   - Integration test plan
   - Benchmark methodology
```

### Success Criteria
- [ ] All ADRs approved
- [ ] Interface specs reviewed
- [ ] Testing strategy defined
- [ ] CI/CD pipeline designed

---

## Phase 2: 核心转写补齐 (Core Implementation)

**Duration**: 6-8 weeks  
**Goal**: Implement all P0 and P1 modules to achieve working CLI

### Milestone 2.1: Foundation (Weeks 1-2)

**Priority**: P0  
**Modules**: options, core

#### options.rs Expansion

| Week | Task | Lines | Deliverable |
|------|------|-------|-------------|
| 1 | Critical options | +500 | Filter, display, search options |
| 1 | Key bindings | +300 | --bind action parsing |
| 2 | Preview options | +400 | --preview-window, --preview |
| 2 | Advanced options | +300 | --tiebreak, --expect, etc. |

**Target**: 678 → 2,200 lines (~55% of Go)

#### core.rs Event Loop

| Week | Task | Deliverable |
|------|------|-------------|
| 1 | EventBox async | Non-blocking event passing |
| 1 | Reader integration | Stdin/command reading |
| 2 | Matcher integration | Parallel matching |
| 2 | Terminal integration | Interactive mode |

**Target**: 223 → 600 lines (~94% of Go)

#### Dependencies
```
options → clap (external)
core → options, reader, matcher, terminal, pattern
```

### Milestone 2.2: Data Pipeline (Weeks 3-4)

**Priority**: P1  
**Modules**: reader, matcher, pattern

#### reader.rs Completion

| Task | Lines | Feature |
|------|-------|---------|
| File walker | +100 | walkdir integration |
| Event polling | +80 | async event notifications |
| Command reload | +60 | --reload support |
| Streaming filter | +50 | Optimization path |

**Target**: 142 → 430 lines (~104% of Go)

#### matcher.rs Parallelism

| Task | Lines | Feature |
|------|-------|---------|
| Rayon integration | +80 | Parallel matching |
| Worker pool | +100 | Partitioned workers |
| Slab allocator | +60 | Per-worker memory |
| Merger cache | +80 | LRU result cache |

**Target**: 149 → 470 lines (~173% of Go)

#### pattern.rs Extended Search

| Task | Lines | Feature |
|------|-------|---------|
| Extended parser | +80 | OR/AND syntax |
| Inverse patterns | +40 | '!' prefix |
| Exact modifiers | +30 | ' / ' prefix |

**Target**: 388 → 540 lines (~108% of Go)

### Milestone 2.3: Terminal & UI (Weeks 5-7)

**Priority**: P1  
**Modules**: terminal, tui, ansi, functions

#### terminal.rs Implementation

| Week | Task | Lines | Feature |
|------|------|-------|---------|
| 5 | Layout engine | +400 | Window calculations |
| 5 | List rendering | +300 | Item display |
| 6 | Preview window | +500 | --preview support |
| 6 | Input handling | +300 | Key event loop |
| 7 | Placeholder expansion | +400 | {} templates |
| 7 | Mouse support | +200 | --no-mouse option |

**Target**: 1,039 → 3,140 lines (~38% of Go)

#### tui/ Platform Support

| Task | Lines | Feature |
|------|-------|---------|
| Unix light | +150 | PTY handling |
| Windows light | +150 | Console API |
| Event mapping | +100 | crossterm → internal |

**Target**: 152 → 550 lines (~11% of Go)

#### ansi.rs Color Processing

| Task | Lines | Feature |
|------|-------|---------|
| 256 color | +100 | Extended colors |
| Truecolor | +80 | 24-bit RGB |
| Sixel/kitty | +50 | Graphics protocols |

**Target**: 69 → 300 lines (~52% of Go)

#### functions.rs Actions

| Task | Lines | Feature |
|------|-------|---------|
| Action bindings | +200 | Key → action map |
| Execute actions | +150 | execute, execute-silent |
| Transform actions | +100 | transform-* |

**Target**: 445 → 900 lines (~82% of Go)

### Milestone 2.4: Supporting Modules (Week 8)

**Priority**: P2  
**Modules**: algo, result/merger, tokenizer, cache, server

| Module | Current | Target | Task |
|--------|---------|--------|------|
| algo/normalize | 12 | 200 | Unicode normalization |
| result | 113 | 350 | Ranking, tiebreakers |
| merger | 105 | 180 | Caching, pass-through |
| tokenizer | 73 | 280 | AWK-style parsing |
| server | 40 | 220 | HTTP API, JSON |

### Phase 2 Deliverables

```markdown
1. Working filter mode: `echo "test" | ./fzf -f "te"`
2. Working interactive mode: `./fzf` (basic)
3. Preview window: `./fzf --preview "echo {}"`
4. Multi-select: `./fzf -m`
5. Key bindings: Custom --bind actions
6. Server mode: `./fzf --listen 6266`

Build: `cargo build --release` succeeds
Tests: `cargo test` passes (target: 80%+ coverage)
```

### Phase 2 Schedule

```
Week 1-2:   [options]        [core]
Week 3-4:   [reader]         [matcher]      [pattern]
Week 5-7:   [terminal]       [tui]          [ansi]    [functions]
Week 8:     [algo] [result] [merger] [tokenizer] [server]
```

---

## Phase 3: 验证与收尾 (Verification & Completion)

**Duration**: 3-4 weeks  
**Goal**: Production-ready release with full test coverage

### Week 9-10: Testing

#### Unit Test Coverage

| Module | Go Tests | Rust Target | Status |
|--------|----------|-------------|--------|
| options | 571 lines | 400 lines | Add edge cases |
| pattern | 318 lines | 300 lines | Fuzz testing |
| algo | 915 lines | 500 lines | Property tests |
| matcher | - | 200 lines | Parallel correctness |
| terminal | 770 lines | 400 lines | Mock terminal |

#### Integration Testing

| Test Suite | Command | Target |
|------------|---------|--------|
| Ruby tests | `ruby test/runner.rb` | 80% pass |
| Filter tests | `ruby test/test_filter.rb` | 100% pass |
| Core tests | `ruby test/test_core.rb` | 80% pass |
| Shell tests | `ruby test/test_shell_integration.rb` | 60% pass |

#### Performance Validation

| Benchmark | Go Baseline | Rust Target |
|-----------|-------------|-------------|
| Filter 1M items | X ops/sec | ≥ 0.8X |
| Memory (1M items) | Y MB | ≤ 1.2Y |
| Startup time | Z ms | ≤ 1.5Z |

### Week 11: CI/CD & Documentation

#### CI/CD Pipeline (.github/workflows/)

```yaml
# ci.yml
- Build on: ubuntu, macos, windows
- Test: cargo test
- Lint: cargo clippy, cargo fmt
- Benchmark: cargo bench
- Cross-compile: release binaries
```

#### Documentation

| Document | Status |
|----------|--------|
| README.md | Update with Rust build |
| BUILD.md | Verify instructions |
| CHANGELOG.md | Migration notes |
| API.md | Rust public API |

### Week 12: Bug Fixes & Release

#### Release Checklist

- [ ] All P0/P1 modules complete
- [ ] Integration tests pass
- [ ] Performance acceptable
- [ ] Documentation complete
- [ ] CI/CD green
- [ ] Security review
- [ ] License compliance

### Phase 3 Deliverables

```markdown
1. Production binary: target/release/fzf
2. Test report: tests/report.md
3. Performance report: benches/report.md
4. Documentation: docs/api/
5. CI/CD: .github/workflows/
6. Release notes: CHANGELOG.md
```

---

## Risk Mitigation

### Technical Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Performance gap | Medium | High | Early benchmarking, SIMD later |
| Terminal bugs | High | Medium | Extensive testing on multiple terminals |
| Memory leaks | Medium | High | Valgrind, Miri, long-running tests |
| Concurrency bugs | Medium | High | Stress testing, loom |

### Schedule Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Scope creep | High | Medium | Strict phase gates |
| Platform issues | Medium | Medium | Unix-first, Windows later |
| SIMD complexity | Low | Medium | Defer to Phase 3 |

---

## Resource Requirements

### Personnel

| Role | Count | Time |
|------|-------|------|
| Rust Lead | 1 | Full-time |
| Rust Developer | 1-2 | Full-time |
| QA Engineer | 1 | 50% (Phase 3) |
| Tech Lead | 1 | 20% (review) |

### Tools

| Category | Tools |
|----------|-------|
| Build | cargo, rustc |
| Testing | cargo test, criterion, proptest |
| Profiling | flamegraph, perf, valgrind |
| CI/CD | GitHub Actions |
| Docs | rustdoc, mdbook |

---

## Success Metrics

### Quantitative

| Metric | Target |
|--------|--------|
| Code coverage | ≥ 80% |
| Test pass rate | ≥ 95% |
| Performance | ≥ 80% of Go |
| Binary size | ≤ 150% of Go |
| Build time | ≤ 2x Go |

### Qualitative

| Criteria | Evaluation |
|----------|------------|
| Feature parity | All common use cases work |
| UX consistency | Same CLI behavior |
| Documentation | Complete API docs |
| Community | Positive feedback |

---

## Appendix: Module Dependency Graph

```
Phase 2.1: Foundation
┌─────────┐     ┌─────────┐
│ options │────→│  core   │
└─────────┘     └────┬────┘
                     │
        ┌────────────┼────────────┐
        ↓            ↓            ↓
   ┌─────────┐  ┌─────────┐  ┌─────────┐
   │ reader  │  │ matcher │  │terminal │
   └────┬────┘  └────┬────┘  └────┬────┘
        │            │            │
        └────────────┼────────────┘
                     ↓
               ┌─────────┐
               │ pattern │
               └────┬────┘
                    ↓
               ┌─────────┐
               │  algo   │
               └─────────┘

Phase 2.2-2.4: Remaining modules
[util] supports all modules
[tui] supports terminal
[ansi] supports terminal
[functions] supports terminal
[result/merger/cache] support matcher
[tokenizer] supports reader
[server] supports core
[history] supports terminal
```

---

## Appendix: Go→Rust Feature Mapping Checklist

| Feature | Go | Rust | Phase |
|---------|-----|------|-------|
| **CLI Options** | 150+ | ~30 | 2.1 |
| **Filter mode** | Full | Partial | 2.1 |
| **Interactive mode** | Full | Stub | 2.1 |
| **Parallel matching** | Full | Single-threaded | 2.2 |
| **File walker** | fastwalk | Missing | 2.2 |
| **Preview window** | Full | Missing | 2.3 |
| **Key bindings** | Full | Partial | 2.3 |
| **ANSI colors** | Full | Basic | 2.3 |
| **Tmux/Zellij** | Full | Missing | 2.4 |
| **Server mode** | Full | Stub | 2.4 |
| **History** | Full | Partial | 2.4 |
| **SIMD** | amd64/arm64 | Missing | 3.0 |
| **Windows support** | Full | Missing | 3.0 |
