Building fzf (Rust)
===================

Build instructions
------------------

### Prerequisites

- Rust 1.70 or above (install via [rustup](https://rustup.rs/))
- Ubuntu/Debian: `sudo apt-get update && sudo apt-get install -y build-essential`

### Using Cargo

```sh
# Clone the repository
git clone <repository-url>
cd fzf

# Build fzf binary in release mode
cargo build --release

# The binary will be available at:
./target/release/fzf

# Run tests
cargo test

# Install locally (optional)
cargo install --path .
```

### Quick Start on Ubuntu

```sh
# 1. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 2. Build
make

# 3. Test
cargo test

# 4. Install to bin/ directory
make install
```

Running tests
-------------

```sh
# Run all Rust unit tests
cargo test

# Run tests with verbose output
cargo test -- --nocapture

# Run specific test
cargo test test_pattern_fuzzy_match

# Run tests in specific module
cargo test algo::tests
cargo test pattern::tests
cargo test functions::tests
```

Verification
------------

```sh
# Verify build
cargo build --release

# Run all tests (should show 42 passed)
cargo test 2>&1 | grep "test result"

# Quick functional verification
./target/release/fzf --version
echo -e "apple\nbanana" | ./target/release/fzf -f "an"
```

Third-party libraries used (Rust)
---------------------------------

- [clap](https://github.com/clap-rs/clap) - Command line argument parser
    - Licensed under [MIT](https://github.com/clap-rs/clap/blob/master/LICENSE-MIT)
- [crossterm](https://github.com/crossterm-rs/crossterm) - Cross-platform terminal manipulation
    - Licensed under [MIT](https://github.com/crossterm-rs/crossterm/blob/master/LICENSE)
- [ratatui](https://github.com/ratatui/ratatui) - Terminal UI library
    - Licensed under [MIT](https://github.com/ratatui/ratatui/blob/main/LICENSE)
- [regex](https://github.com/rust-lang/regex) - Regular expressions
    - Licensed under [MIT](https://github.com/rust-lang/regex/blob/master/LICENSE-MIT)
- [rayon](https://github.com/rayon-rs/rayon) - Data parallelism
    - Licensed under [MIT](https://github.com/rayon-rs/rayon/blob/master/LICENSE-MIT)

License
-------

[MIT](LICENSE)
