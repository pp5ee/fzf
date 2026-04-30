# fzf Go→Rust Migration: Scope & Verification Path

## Executive Summary

**Code Scanning Status**: ✅ **COMPLETE**

Based on comprehensive analysis (AC-1 through AC-8), this document defines:
1. **Priority Migration Scope** - What will be implemented and in what order
2. **Out of Scope** - What is explicitly excluded or deferred
3. **Verification Path** - How each phase and the final deliverable will be validated

**Current State**:
- Go codebase: 29,695 lines across 79 files
- Rust codebase: 5,564 lines across 29 files (18.7% complete)
- Critical path identified: options → core → reader → matcher → terminal

---

## 1. Priority Migration Scope

### 1.1 Phase-Based Scope Definition

Based on the module analysis (AC-3) and phased plan (AC-4), the migration scope is organized by priority:

```
Phase 0 (Week 1):         ┌───────────────────┐
Current State Analysis    │ Establish baselines   │
                          └───────────────────┘

Phase 1 (Weeks 2-3):      ┌───────────────────┐
Architecture & Design     │ ADRs + Interfaces     │
                          └───────────────────┘

Phase 2 (Weeks 4-10):     ┌───────────────────┐
Core Implementation       │ P0 + P1 Modules       │
                          └───────────────────┘

Phase 3 (Weeks 11-14):    ┌───────────────────┐
Verification & Release    │ Testing + Polish      │
                          └───────────────────┘
```

### 1.2 In-Scope Modules (Priority Order)

#### P0 - Critical (Must Have for MVP)

| Module | Go Lines | Priority | Justification |
|--------|----------|----------|---------------|
| **options** | 4,039 | 1st | CLI parsing foundation - all features depend on this |
| **core** | 640 | 2nd | Event loop coordination - blocks all other modules |
| **reader** | 413 | 3rd | Input pipeline - required for basic operation |
| **matcher** | 271 | 4th | Search engine - core fuzzy finding functionality |

**P0 Scope Details**:
- **options**: ~40 critical CLI options (filter, exact, case, nth, etc.)
- **core**: Event loop with tokio, filter mode, basic interactive mode
- **reader**: Stdin reading, command execution, event polling
- **matcher**: Parallel matching with rayon, V1/V2 algorithms, result ranking

**P0 Acceptance Criteria**:
```
✅ Binary builds on Ubuntu 22.04
✅ Filter mode: `echo "test" | fzf -f "te"` works
✅ Interactive mode launches without crash
✅ Exit codes match Go (0=match, 1=no-match, 2=error)
✅ 20+ Ruby filter tests passing
```

#### P1 - High (Required for Feature Parity)

| Module | Go Lines | Priority | Justification |
|--------|----------|----------|---------------|
| **terminal** | 8,243 | 5th | UI rendering - major user-facing feature |
| **functions** | 1,100 | 6th | Action handlers - key bindings and interactions |
| **tui** | 4,812 | 7th | Terminal abstraction - platform support |
| **ansi** | 578 | 8th | Color processing - visual appearance |

**P1 Scope Details**:
- **terminal**: Layout engine, preview window, placeholder expansion
- **functions**: Key binding actions, execute/transform actions
- **tui**: crossterm integration, platform-specific terminal handling
- **ansi**: Color code parsing, 256/truecolor support

**P1 Acceptance Criteria**:
```
✅ Preview window: `fzf --preview "echo {}"` works
✅ Key bindings: Custom --bind actions work
✅ Multi-select: Tab/shift-tab selection works
✅ ANSI colors: Color codes rendered correctly
✅ 80+ Ruby core tests passing
```

#### P2 - Medium (Polish & Performance)

| Module | Go Lines | Priority | Justification |
|--------|----------|----------|---------------|
| **algo** | 2,622 | 9th | SIMD optimizations - performance enhancement |
| **result/merger** | 644 | 10th | Result handling - scoring/caching |
| **tokenizer** | 338 | 11th | Field extraction - --nth option |
| **pattern** | 501 | 12th | Extended search - advanced queries |

**P2 Scope Details**:
- **algo**: SIMD port (amd64/arm64) using Rust intrinsics
- **result**: Tiebreaker implementation, caching
- **tokenizer**: AWK-style delimiter parsing
- **pattern**: Extended search syntax (OR/AND)

**P2 Acceptance Criteria**:
```
✅ Performance within 20% of Go on filter benchmarks
✅ Extended search: `pattern1 | pattern2` works
✅ --nth option: Field extraction works
✅ All tiebreaker criteria implemented
```

#### P3 - Low (Deferred to Post-MVP)

| Module | Go Lines | Priority | Justification |
|--------|----------|----------|---------------|
| **server** | 276 | Post-MVP | HTTP API - advanced feature |
| **history** | 95 | Post-MVP | Persistent history - nice to have |
| **cache** | 98 | Post-MVP | Chunk caching - optimization |

**P3 Decision**: These modules have Rust stubs but incomplete implementations. They will be completed after P0/P1/P2 if time permits.

### 1.3 Scope Matrix

```
Module          P0      P1      P2      P3      Out
---------------------------------------------------
options         ████████░░      ░░      ░░      
░░
core            ████████░░      ░░      ░░      
░░
reader          ████████░░      ░░      ░░      
░░
matcher         ████████░░      ░░      ░░      
░░
terminal        ░░      ████████░░      ░░      
░░
functions       ░░      ████████░░      ░░      
░░
tui             ░░      ████████░░      ░░      
░░
ansi            ░░      ░░      ████████░░      
░░
algo            ░░      ░░      ████████░░      
░░
result/merger   ░░      ░░      ████████░░      
░░
tokenizer       ░░      ░░      ████████░░      
░░
server          ░░      ░░      ░░      ██████░░
history         ░░      ░░      ░░      ██████░░
cache           ░░      ░░      ░░      ██████░░
protector       ░░      ░░      ░░      ░░      ████
platform/win    ░░      ░░      ░░      ░░      ████

Legend: ████ = In Scope    ░░ = Out of Scope for this phase
```

---

## 2. Out of Scope

### 2.1 Explicitly Excluded

| Item | Reason | Alternative |
|------|--------|-------------|
| **OpenBSD Protector** | OpenBSD-specific, minimal user impact | Skip entirely |
| **Windows-specific code** | Resource constraints, Unix primary | Basic cross-platform only |
| **Winpty support** | Legacy Windows terminal | Not needed for modern Windows |
| **SIMD Assembly** | Can use Rust intrinsics instead | Port to Rust SIMD when needed |

### 2.2 Deferred to Future Releases

| Item | Deferred To | Trigger |
|------|-------------|---------|
| **Tmux popup integration** | v0.73.0 | User demand |
| **Zellij popup integration** | v0.73.0 | User demand |
| **Server mode (HTTP API)** | v0.73.0 | Time permitting |
| **History persistence** | v0.73.0 | Time permitting |
| **Advanced preview features** | v0.74.0 | Feedback-driven |
| **SIMD optimizations** | v0.74.0 | Performance analysis |

### 2.3 Scope Decisions Justification

**Why P3 modules are deferred**:
1. **Server mode**: Advanced feature, not required for basic fzf usage
2. **History**: Has Rust stub, but persistence can be added later
3. **Cache**: Performance optimization, correctness first

**Why platform code is limited**:
1. **Unix/Linux** is primary target (95%+ usage based on downloads)
2. **Windows** support via crossterm (basic) - advanced console features deferred
3. **OpenBSD pledge** is security theater for this use case

---

## 3. Verification Path

### 3.1 Verification Overview

```
Verification Pipeline:

Source Code          Unit Tests         Integration Tests       Performance
     ↓                    ↓                     ↓                    ↓
┌─────────┐         ┌─────────┐          ┌─────────┐         ┌─────────┐
│ clippy   │         │ cargo   │          │ Ruby    │         │ criterion │
│ fmt      │   →    │ test    │    →     │ tests   │    →    │ benches   │
│ audit    │         │ tarpaulin│         │ (Go ref) │        │ valgrind  │
└─────────┘         └─────────┘          └─────────┘         └─────────┘
     ┌─────────┐         ┌─────────┐          ┌─────────┐         ┌─────────┐
     │ Quality │         │ Coverage ≥80% │        │ 90% pass  │        │ No regress│
     └─────────┘         └─────────┘          └─────────┘         └─────────┘
```

### 3.2 Phase-Specific Verification

#### Phase 0 Verification (Week 1)

| Artifact | Verification Method | Acceptance Criteria |
|----------|---------------------|---------------------|
| Project Structure | Document review | Complete file inventory |
| Gap Matrix | Automated line count | Accuracy > 95% |
| Build Baseline | `make && cargo build` | Both binaries build |
| Test Baseline | `cargo test` | Current state recorded |

**Phase 0 Gate**: All baselines documented and reproducible.

#### Phase 1 Verification (Weeks 2-3)

| Artifact | Verification Method | Acceptance Criteria |
|----------|---------------------|---------------------|
| ADRs | Team review | All decisions documented |
| Interface Specs | Code review | Traits compile |
| CI/CD | Pipeline execution | Green builds on PRs |
| Type Mapping | Review + test | Core types defined |

**Phase 1 Gate**: All architecture decisions approved, CI operational.

#### Phase 2 Verification (Weeks 4-10)

| Module | Unit Tests | Integration Tests | Benchmarks |
|--------|------------|-------------------|------------|
| options | 50 tests | 10 Ruby tests | N/A |
| core | 20 tests | 20 Ruby tests | Startup time |
| reader | 15 tests | 10 Ruby tests | Throughput |
| matcher | 25 tests | 20 Ruby tests | Parallel speedup |
| terminal | 30 tests | 40 Ruby tests | Render time |
| functions | 20 tests | 15 Ruby tests | N/A |

**Phase 2 Gates**:
- Week 4 (Milestone 2.1): options + core unit tests pass
- Week 6 (Milestone 2.2): reader + matcher integration tests pass
- Week 8 (Milestone 2.3): terminal + functions integration tests pass
- Week 10 (Milestone 2.4): All P0/P1 tests passing

#### Phase 3 Verification (Weeks 11-14)

| Test Suite | Target | Minimum | Method |
|------------|--------|---------|--------|
| Rust unit tests | 300+ | 250 | `cargo test` |
| Code coverage | 80% | 70% | `cargo tarpaulin` |
| Ruby filter tests | 45/50 | 40/50 | `ruby test/rust_filter.rb` |
| Ruby core tests | 80/100 | 70/100 | `ruby test/rust_core.rb` |
| Ruby shell tests | 15/15 | 15/15 | `ruby test/rust_shell.rb` |
| Performance | ±20% | ±30% | `cargo bench` |

**Phase 3 Gates**:
- Week 11: Coverage ≥ 70%, no critical bugs
- Week 12: Coverage ≥ 80%, 90% Ruby tests passing
- Week 13: Release candidate, beta testing
- Week 14: Final release

### 3.3 Continuous Verification

#### Pre-commit Checks (Every PR)

```yaml
# .github/workflows/pr.yml
jobs:
  verify:
    steps:
      - run: cargo fmt -- --check
      - run: cargo clippy -- -D warnings
      - run: cargo test
      - run: cargo test --release
      - run: ruby test/rust_verify.rb
```

#### Nightly Checks

```yaml
# .github/workflows/nightly.yml
jobs:
  full-verification:
    steps:
      - run: cargo test --all-features
      - run: cargo tarpaulin --fail-under 70
      - run: cargo bench
      - run: ruby test/rust_filter.rb
      - run: ruby test/rust_core.rb
```

### 3.4 Final Acceptance Criteria

#### Functional Requirements

| Requirement | Verification | Criteria |
|-------------|--------------|----------|
| Filter mode | Ruby tests | 40/50 tests passing |
| Interactive mode | Manual + Ruby | No crashes, basic features work |
| Preview window | Ruby tests | 20/25 tests passing |
| Key bindings | Manual | All documented bindings work |
| Shell integration | Ruby tests | 15/15 tests passing |

#### Performance Requirements

| Metric | Go Baseline | Rust Target | Verification |
|--------|-------------|-------------|--------------|
| Filter 1M items | 0.45s | < 0.55s | `cargo bench` |
| Memory (1M items) | 85 MB | < 100 MB | `valgrind` |
| Startup time | 15ms | < 20ms | Manual timing |
| Parallel speedup | 6.5x | > 5x | `cargo bench` |

#### Compatibility Requirements

| Aspect | Verification | Criteria |
|--------|--------------|----------|
| CLI options | Comparison test | 40+ options match |
| Exit codes | Ruby tests | 100% match |
| Output format | Diff comparison | Identical |
| Shell scripts | Direct comparison | Byte-identical |

#### Quality Requirements

| Aspect | Target | Verification |
|--------|--------|--------------|
| Code coverage | ≥ 80% | `cargo tarpaulin` |
| Clippy warnings | 0 | `cargo clippy` |
| Documentation | 100% public APIs | `cargo doc` |
| Unsafe code | Minimal + audited | Manual review |

---

## 4. Risk-Based Scope Adjustments

### 4.1 Scope Reduction Triggers

If schedule pressure occurs, scope will be reduced in this order:

1. **First**: P3 modules (server, history) - already deferred
2. **Second**: P2 features within P1 modules (advanced preview options)
3. **Third**: P1 modules (terminal polish, some key bindings)
4. **Last**: P0 core functionality (never reduced)

### 4.2 Scope Expansion Triggers

If ahead of schedule, consider adding:

1. **SIMD optimizations** (from P2 to earlier)
2. **Additional platform support** (Windows enhancements)
3. **Deferred P3 modules** (server mode)

### 4.3 Verification Adjustment

If test coverage cannot reach 80%:
- Minimum acceptable: 70% for release
- Must have 100% coverage on: matcher, reader, pattern
- Can accept lower on: terminal (UI code), options (boilerplate)

---

## 5. Success Definition

### 5.1 Definition of Done

A module is considered migrated when:

1. [ ] All planned features implemented
2. [ ] Unit tests passing (≥ 80% coverage)
3. [ ] Integration tests passing (Ruby tests)
4. [ ] No clippy warnings
5. [ ] Documentation complete
6. [ ] Performance within 20% of Go
7. [ ] Code review approved

### 5.2 Project Success Criteria

| Criterion | Target | Measurement |
|-----------|--------|-------------|
| Feature parity | 90% | Ruby test pass rate |
| Performance | ±20% | Benchmark comparison |
| Code quality | A grade | Clippy + audit |
| Test coverage | ≥ 80% | Tarpaulin report |
| User adoption | 50% beta | Download metrics |

### 5.3 Failure Criteria (Stop Conditions)

Migration will be halted if:
1. Performance > 2x slower than Go after optimization
2. Critical security vulnerabilities found
3. Unresolvable architectural blockers
4. Team capacity reduced > 50%

---

## 6. Summary

### 6.1 Scope Recap

| Category | Modules | Lines | Priority |
|----------|---------|-------|----------|
| **P0 - Critical** | options, core, reader, matcher | 5,363 | Immediate |
| **P1 - High** | terminal, functions, tui, ansi | 14,733 | Required |
| **P2 - Medium** | algo, result, merger, tokenizer, pattern | 4,105 | Polish |
| **P3 - Low** | server, history, cache | 469 | Deferred |
| **Out** | protector, platform/win | ~500 | Excluded |

**Total In-Scope**: 24,301 lines (82% of Go codebase)

### 6.2 Verification Recap

| Phase | Key Verification | Gate |
|-------|------------------|------|
| 0 | Baselines documented | Reproducible |
| 1 | ADRs approved, CI green | Design locked |
| 2 | Tests passing per module | Feature complete |
| 3 | 90% Ruby tests, 80% coverage | Release ready |

### 6.3 Final Acceptance

The migration is **ACCEPTED** when:

```
✅ All P0 modules complete (options, core, reader, matcher)
✅ All P1 modules complete (terminal, functions, tui, ansi)
✅ 90%+ Ruby integration tests passing
✅ 80%+ code coverage
✅ Performance within 20% of Go
✅ Documentation complete
✅ CI/CD operational
```

Estimated completion: **14 weeks** from Phase 0 start.
