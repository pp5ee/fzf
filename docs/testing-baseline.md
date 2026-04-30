# fzf Rust Migration: Testing Baseline & Validation Strategy

## Executive Summary

| Category | Go | Rust (Current) | Target |
|----------|-----|----------------|--------|
| **Unit Tests** | 19 test files (~3,500 lines) | 7 inline test modules (~800 lines) | Match Go coverage |
| **Integration Tests** | 8 Ruby files (~7,000 lines) | 1 basic file (95 lines) | Port critical tests |
| **CI/CD** | None configured | None configured | Add GitHub Actions |
| **Benchmarks** | Go bench | criterion stub | Performance parity |

**Strategy**: Leverage existing Ruby integration tests as the primary validation baseline while augmenting with Rust-native unit tests.

---

## 1. Existing Test Infrastructure Analysis

### 1.1 Go Unit Tests (`*_test.go`)

| Test File | Lines | Coverage Focus | Rust Status |
|-----------|-------|----------------|-------------|
| `algo_test.go` | 915 | Algorithm correctness | ✅ Partial (tests.rs) |
| `options_test.go` | 571 | CLI parsing | ❌ Missing |
| `pattern_test.go` | 318 | Pattern matching | ❌ Missing |
| `terminal_test.go` | 770 | Terminal I/O | ❌ Missing |
| `tcell_test.go` | 473 | TUI backend | ❌ Missing |
| `merger_test.go` | 104 | Result merging | ❌ Missing |
| `result_test.go` | 274 | Scoring/ranking | ❌ Missing |
| `reader_test.go` | 68 | Input reading | ❌ Missing |
| `item_test.go` | 23 | Item model | ❌ Missing |
| `chunklist_test.go` | 116 | Chunk storage | ❌ Missing |
| `history_test.go` | 67 | History persistence | ❌ Missing |
| `eventbox_test.go` | 60 | Event coordination | ❌ Missing |
| `ansi_test.go` | 559 | ANSI processing | ❌ Missing |
| `light_test.go` | 352 | Light terminal | ❌ Missing |
| `tokenizer_test.go` | 125 | Tokenization | ❌ Missing |
| `util_test.go` | 163 | Utilities | ❌ Missing |
| `chars_test.go` | 131 | Char handling | ❌ Missing |
| `slab_test.go` | 0 (in algo) | Memory allocation | ❌ Missing |
| **Total** | **~4,219 lines** | **Comprehensive** | **~10% covered** |

### 1.2 Ruby Integration Tests

| Test File | Lines | Test Type | Reuse for Rust |
|-----------|-------|-----------|----------------|
| `test_filter.rb` | 529 | Non-interactive (filter mode) | **Primary baseline** |
| `test_core.rb` | 2,500 | Interactive core features | Secondary baseline |
| `test_layout.rb` | 1,700 | UI layout | Tertiary baseline |
| `test_preview.rb` | 850 | Preview window | Tertiary baseline |
| `test_shell_integration.rb` | 1,200 | Shell scripts | **Reuse scripts** |
| `test_exec.rb` | 500 | Command execution | Secondary baseline |
| `test_server.rb` | 361 | HTTP server | Tertiary baseline |
| `test_raw.rb` | 277 | Raw mode | Secondary baseline |
| **Total** | **~7,000 lines** | **End-to-end** | **Adapt for Rust** |

### 1.3 Rust Tests (Current)

| Test Module | Lines | Coverage | Status |
|-------------|-------|----------|--------|
| `algo/tests.rs` | 206 | Algorithm tests | ✅ Basic |
| `functions/tests.rs` | 141 | Action handlers | ✅ Basic |
| `pattern.rs (inline)` | ~100 | Pattern matching | ⚠️ Incomplete |
| `options.rs (inline)` | ~50 | CLI parsing | ⚠️ Minimal |
| `item.rs (inline)` | ~30 | Item model | ⚠️ Minimal |
| `matcher.rs (inline)` | ~50 | Matching | ⚠️ Minimal |
| `merger.rs (inline)` | ~30 | Merging | ⚠️ Minimal |
| **Total** | **~607 lines** | **~17% of Go** | **Needs expansion** |

---

## 2. Validation Baseline Strategy

### 2.1 Primary Baseline: Ruby Filter Tests

**Rationale**: `test_filter.rb` tests non-interactive filter mode which is:
- Easiest to port (no terminal required)
- Core functionality (fuzzy matching)
- Already has Rust stub (`rust_verify.rb`)

**Implementation**:

```ruby
# test/rust_baseline.rb - Extended from rust_verify.rb
#!/usr/bin/env ruby
# frozen_string_literal: true

require 'minitest/autorun'
require_relative 'lib/common'

BASE = File.expand_path('..', __dir__)
FZF_GO = "#{BASE}/bin/fzf-go"  # Reference binary
FZF_RS = "#{BASE}/target/release/fzf"  # Rust binary

class TestRustBaseline < Minitest::Test
  def setup
    skip "Rust binary not found" unless File.exist?(FZF_RS)
    skip "Go binary not found" unless File.exist?(FZF_GO)
  end

  # Phase 0: Behavior parity tests
  def test_basic_filter_matches_go
    input = "apple\nbanana\ncherry"
    go_result = `echo "#{input}" | #{FZF_GO} -f "an"`.lines(chomp: true)
    rs_result = `echo "#{input}" | #{FZF_RS} -f "an"`.lines(chomp: true)

    assert_equal go_result.sort, rs_result.sort,
                 "Filter results should match between Go and Rust"
  end

  def test_exit_codes_match_go
    # Match case
    `echo "test" | #{FZF_GO} -f "test"`
    go_exit = $?.exitstatus

    `echo "test" | #{FZF_RS} -f "test"`
    rs_exit = $?.exitstatus

    assert_equal go_exit, rs_exit, "Exit codes should match"

    # No match case
    `echo "foo" | #{FZF_GO} -f "bar" 2>/dev/null`
    go_exit = $?.exitstatus

    `echo "foo" | #{FZF_RS} -f "bar" 2>/dev/null`
    rs_exit = $?.exitstatus

    assert_equal go_exit, rs_exit, "Exit codes for no-match should match"
  end

  # Phase 1: Comprehensive filter tests (adapted from test_filter.rb)
  def test_exact_match
    skip "Not yet implemented" unless rust_supports?(:exact)

    input = (1..123).to_a.join("\n")
    go_result = `echo "#{input}" | #{FZF_GO} -f 13 -e`.lines.length
    rs_result = `echo "#{input}" | #{FZF_RS} -f 13 -e`.lines.length

    assert_equal go_result, rs_result
  end

  def test_or_operator
    skip "Not yet implemented" unless rust_supports?(:extended)

    input = (1..10).to_a.join("\n")
    go_result = `echo "#{input}" | #{FZF_GO} -f "1 | 5"`.lines(chomp: true)
    rs_result = `echo "#{input}" | #{FZF_RS} -f "1 | 5"`.lines(chomp: true)

    assert_equal go_result.sort, rs_result.sort
  end

  # Phase 2: Performance baseline
  def test_performance_regression
    skip "Performance testing in Phase 2"

    input = (1..1_000_000).map { |i| "item-#{i}" }.join("\n")

    go_time = Benchmark.measure {
      `echo "#{input}" | #{FZF_GO} -f "500000"`
    }.real

    rs_time = Benchmark.measure {
      `echo "#{input}" | #{FZF_RS} -f "500000"`
    }.real

    # Rust should be within 2x of Go (initial target)
    assert_operator rs_time, :<, go_time * 2,
                    "Rust should not be more than 2x slower than Go"
  end
end
```

### 2.2 Secondary Baseline: Extended Ruby Tests

**Phase 1**: Port selected tests from `test_core.rb`:
- Key binding tests (non-interactive)
- Query handling
- Basic selection

**Phase 2**: Port from `test_exec.rb`:
- Command execution
- Placeholder expansion

**Phase 3**: Port from `test_preview.rb`:
- Preview window (once implemented)

### 2.3 Tertiary Baseline: Shell Integration

**Reuse as-is**: The shell scripts in `shell/` are language-agnostic:
- `key-bindings.bash`
- `key-bindings.zsh`
- `key-bindings.fish`
- `completion.*`

**Validation**:
```ruby
# test/shell_integration_rust.rb
def test_shell_integration_outputs_match
  %w[bash zsh fish].each do |shell|
    go_output = `#{FZF_GO} --#{shell}`
    rs_output = `#{FZF_RS} --#{shell}`

    assert_equal go_output, rs_output,
                 "#{shell} integration should be identical"
  end
end
```

---

## 3. Minimal Test Set Requirements

### 3.1 Phase 0: Baseline (Week 1)

**Unit Tests** (Rust native):
```rust
// src/options.rs - Minimal CLI tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_options_parse() {
        let opts = parse_options(&["-f", "test"]).unwrap();
        assert_eq!(opts.filter, Some("test".to_string()));
    }

    #[test]
    fn test_exact_flag() {
        let opts = parse_options(&["-e", "-f", "test"]).unwrap();
        assert!(opts.exact);
    }
}
```

**Integration Tests** (Ruby):
```ruby
# test/rust_phase0.rb
class TestPhase0 < Minitest::Test
  def test_binary_exists
    assert File.exist?(FZF_RS), "Rust binary should exist"
  end

  def test_version_format
    result = `#{FZF_RS} --version`
    assert_match(/^\d+\.\d+\.\d+/, result)
  end

  def test_filter_basic
    result = `echo "test" | #{FZF_RS} -f "test"`
    assert_equal "test", result.chomp
  end
end
```

### 3.2 Phase 1: Core Functionality (Weeks 2-3)

**Required Unit Tests**:
| Module | Test Count | Focus |
|--------|------------|-------|
| options | 20 | CLI parsing edge cases |
| pattern | 30 | Fuzzy, exact, extended modes |
| matcher | 15 | Parallel matching correctness |
| reader | 10 | Stdin, file, command input |
| algo | 50 | V1/V2 scoring consistency |

**Required Integration Tests**:
| Feature | Test Count | Source |
|---------|------------|--------|
| Filter mode | 20 | Adapt from test_filter.rb |
| Exit codes | 10 | New tests |
| Shell integration | 6 | Reuse existing |

### 3.3 Phase 2: Terminal (Weeks 5-7)

**Challenge**: Interactive terminal tests require tmux.

**Strategy**:
```ruby
# test/interactive_rust.rb
class TestInteractiveRust < TestInteractive
  # Inherit from existing test framework
  # Reuse tmux helpers from lib/common.rb

  def test_basic_interactive
    tmux.send_keys %(seq 100 | #{FZF_RS}), :Enter
    tmux.until { |lines| assert_equal 100, lines.match_count }

    tmux.send_keys '50'
    tmux.until { |lines| assert lines.match_count < 100 }
  end
end
```

**Mock Terminal Tests** (Rust native):
```rust
// tests/terminal_mock.rs
use fzf::terminal::Terminal;
use crossterm::event::{Event, KeyCode};

#[tokio::test]
async fn test_key_event_handling() {
    let mut term = Terminal::new_test();

    // Simulate key press
    term.handle_event(Event::Key(KeyCode::Char('a').into())).await;

    assert_eq!(term.query(), "a");
}
```

### 3.4 Phase 3: Full Coverage (Weeks 9-12)

**Target Metrics**:
| Metric | Go Baseline | Rust Target |
|--------|-------------|-------------|
| Unit test coverage | ~70% | ≥ 70% |
| Integration test pass | 100% | ≥ 90% |
| Ruby test adaptation | 7,000 lines | ≥ 4,000 lines |
| Performance regression | Baseline | ≤ 20% slower |

---

## 4. Test Infrastructure Setup

### 4.1 CI/CD Pipeline (`.github/workflows/rust.yml`)

```yaml
name: Rust

on: [push, pull_request]

jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest]
        rust: [stable]

    steps:
    - uses: actions/checkout@v3

    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable

    - name: Install Go (for comparison tests)
      uses: actions/setup-go@v4
      with:
        go-version: '1.23'

    - name: Build Go reference
      run: make bin/fzf-go

    - name: Build Rust
      run: cargo build --release

    - name: Run Rust unit tests
      run: cargo test --verbose

    - name: Run baseline tests
      run: |
        cp target/release/fzf bin/fzf
        ruby test/rust_baseline.rb

    - name: Run filter tests (Phase 1+)
      if: matrix.phase >= 1
      run: ruby test/rust_filter.rb

    - name: Run core tests (Phase 2+)
      if: matrix.phase >= 2
      run: ruby test/rust_core.rb

    - name: Performance benchmarks
      run: cargo bench

  coverage:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    - uses: dtolnay/rust-toolchain@stable
      with:
        components: llvm-tools-preview
    - name: Generate coverage
      run: |
        cargo install cargo-llvm-cov
        cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
    - name: Upload coverage
      uses: codecov/codecov-action@v3
      with:
        files: lcov.info
```

### 4.2 Test Helpers (Rust)

```rust
// src/test_helpers.rs
#[cfg(test)]
pub mod helpers {
    use std::process::Command;

    /// Run fzf with input and return output
    pub fn run_filter(input: &str, args: &[&str]) -> Vec<String> {
        let mut cmd = Command::new("cargo");
        cmd.args(["run", "--", "-f"])
            .args(args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped());

        let mut child = cmd.spawn().unwrap();

        // Write input
        use std::io::Write;
        child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();

        // Read output
        let output = child.wait_with_output().unwrap();
        String::from_utf8(output.stdout)
            .unwrap()
            .lines()
            .map(|s| s.to_string())
            .collect()
    }

    /// Compare Rust output with expected
    pub fn assert_filter_matches(input: &str, pattern: &str, expected: &[&str]) {
        let results = run_filter(input, &[pattern]);
        assert_eq!(results, expected);
    }
}
```

### 4.3 Test Data

```
test/
├── data/
│   ├── simple.txt          # Basic test input
│   ├── unicode.txt         # Unicode test cases
│   ├── ansi.txt            # ANSI color codes
│   ├── long_lines.txt      # Performance test
│   └── special_chars.txt   # Edge cases
├── fixtures/
│   └── expected/           # Expected outputs
└── scripts/
    └── generate_fixtures.rb
```

---

## 5. Test Migration Roadmap

### Week 1: Phase 0 - Baseline
- [ ] Create `test/rust_baseline.rb` (extended from `rust_verify.rb`)
- [ ] Add unit tests for `options` module
- [ ] Set up GitHub Actions workflow
- [ ] Create test data fixtures

### Weeks 2-3: Phase 1 - Core
- [ ] Port 20 filter tests from `test_filter.rb`
- [ ] Add unit tests for `pattern` module
- [ ] Add unit tests for `matcher` module
- [ ] Add unit tests for `reader` module
- [ ] Create Go/Rust comparison harness

### Weeks 5-7: Phase 2 - Terminal
- [ ] Create mock terminal testing framework
- [ ] Port 10 interactive tests from `test_core.rb`
- [ ] Add integration tests for preview window
- [ ] Test key binding actions

### Weeks 9-12: Phase 3 - Full Coverage
- [ ] Achieve 70% unit test coverage
- [ ] Port remaining Ruby tests
- [ ] Add performance regression tests
- [ ] Add stress tests (1M+ items)
- [ ] Documentation complete

---

## 6. Compatibility Testing

### 6.1 CLI Compatibility Matrix

| Option | Go | Rust | Test |
|--------|-----|------|------|
| `-f, --filter` | ✅ | ✅ | ✅ |
| `-e, --exact` | ✅ | ✅ | ✅ |
| `-i, --ignore-case` | ✅ | Partial | ⚠️ |
| `--extended` | ✅ | Partial | ⚠️ |
| `--preview` | ✅ | ❌ | ❌ |
| `--bind` | ✅ | ❌ | ❌ |
| `--height` | ✅ | ❌ | ❌ |
| `--layout` | ✅ | ❌ | ❌ |

### 6.2 Exit Code Compatibility

| Scenario | Exit Code | Test |
|----------|-----------|------|
| Match found | 0 | ✅ |
| No match | 1 | ✅ |
| Error | 2 | ⚠️ |
| SIGINT | 130 | ⚠️ |
| Multi-select empty | 1 | ❌ |

### 6.3 Output Format Compatibility

| Format | Go | Rust | Test |
|--------|-----|------|------|
| Plain text | ✅ | ✅ | ✅ |
| With ANSI | ✅ | Partial | ⚠️ |
| `--print0` | ✅ | ✅ | ✅ |
| Multi-select | ✅ | ❌ | ❌ |

---

## 7. Missing Test Coverage (To Be Added)

### 7.1 Critical Gaps

| Module | Gap | Priority |
|--------|-----|----------|
| options | CLI edge cases | P0 |
| pattern | Extended search syntax | P0 |
| matcher | Parallel correctness | P0 |
| terminal | Mock testing framework | P1 |
| algo | SIMD correctness (future) | P2 |
| server | HTTP API | P2 |

### 7.2 New Tests Required

```rust
// Property-based testing with proptest
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn fuzzy_match_never_panic(
            pattern in "[a-zA-Z0-9]*",
            text in "[a-zA-Z0-9]*"
        ) {
            // Should never panic regardless of input
            let _ = fuzzy_match_v2(&pattern.chars().collect::<Vec<_>>(),
                                   &text.chars().collect::<Vec<_>>(),
                                   Case::Smart, false, true);
        }
    }
}

// Fuzz testing
#[cfg(fuzzing)]
mod fuzz_tests {
    use libfuzzer_sys::fuzz_target;

    fuzz_target!(|data: &[u8]| {
        if let Ok(s) = std::str::from_utf8(data) {
            let _ = Pattern::parse(s);
        }
    });
}
```

---

## 8. Test Deliverables

| Deliverable | Format | Phase | Status |
|-------------|--------|-------|--------|
| Test baseline report | Markdown | 0 | 🔄 |
| Ruby baseline tests | Ruby | 0 | 🔄 |
| Unit test suite | Rust | 1-2 | 🔄 |
| Integration test suite | Ruby | 2-3 | 🔄 |
| CI/CD pipeline | YAML | 0-1 | 🔄 |
| Performance benchmarks | Rust | 2 | 🔄 |
| Coverage report | LCOV | 3 | 🔄 |
| Test documentation | Markdown | 3 | 🔄 |

**Legend**: ✅ Complete | 🔄 In Progress | ❌ Not Started | ⚠️ Partial

---

## 9. Success Criteria

### Quantitative

- [ ] Unit test coverage ≥ 70%
- [ ] Ruby test pass rate ≥ 90%
- [ ] No regressions in filter mode tests
- [ ] Performance within 20% of Go

### Qualitative

- [ ] Test suite runs in CI on every PR
- [ ] Test failures block merge
- [ ] Coverage reports published
- [ ] Documentation complete
