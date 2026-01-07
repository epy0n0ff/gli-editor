# Quickstart Guide: Gitleaks Ignore File Editor

**Feature**: 001-gitleaksignore-editor
**Date**: 2025-12-25
**Target Audience**: Developers implementing the feature

## Purpose

This guide provides step-by-step instructions for setting up the development environment, building the project, running tests, and contributing code for the Gitleaks Ignore File Editor (`gli-editor`).

---

## Prerequisites

### Required Tools

- **Rust**: 1.75 or later (latest stable recommended)
- **Cargo**: Comes with Rust installation
- **Git**: For version control
- **Terminal**: Any modern terminal (iTerm2, Terminal.app, Windows Terminal, etc.)

### Recommended Tools

- **Rust Analyzer**: IDE/editor plugin for Rust development
- **cargo-watch**: Auto-rebuild on file changes (`cargo install cargo-watch`)
- **cargo-nextest**: Faster test runner (`cargo install cargo-nextest`)

### Check Your Environment

```bash
# Verify Rust installation
rustc --version  # Should show 1.75.0 or later
cargo --version  # Should match Rust version

# Verify Git
git --version    # Any recent version is fine
```

---

## Quick Start (5 Minutes)

### 1. Clone and Build

```bash
# Clone the repository
git clone https://github.com/epy0n0ff/gli-editor.git
cd gli-editor

# Build the project
cargo build

# Run the application
cargo run -- --help
```

### 2. Run Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_pattern_parsing
```

### 3. Try It Out

```bash
# Create a test file
cat > test.gitleaksignore <<EOF
# Test comment
cd5226711335c68be1e720b318b7bc3135a30eb2:cmd/file.go:sidekiq-secret:23

9f067a02a5efa7110da117e80e8ea58d26847e70:docs/index.md:generic-api-key:85
EOF

# Open it with gli-editor
cargo run -- test.gitleaksignore
```

---

## Project Structure

```
gli-editor/
├── src/
│   ├── main.rs              # Entry point & CLI argument parsing
│   ├── app.rs               # Application state & main loop
│   ├── ui/                  # Ratatui UI components
│   │   ├── mod.rs
│   │   ├── viewer.rs        # Line viewing widget
│   │   ├── editor.rs        # Inline editing widget
│   │   ├── navigation.rs    # Navigation controls
│   │   └── styles.rs        # Syntax highlighting styles
│   ├── core/                # Business logic (library layer)
│   │   ├── mod.rs
│   │   ├── file_reader.rs   # File I/O operations
│   │   ├── line_parser.rs   # Pattern type detection
│   │   ├── editor.rs        # Edit operation management
│   │   └── backup.rs        # Backup file handling
│   ├── models/              # Data structures
│   │   ├── mod.rs
│   │   ├── line.rs          # Line and LineRange types
│   │   ├── pattern.rs       # PatternType enum
│   │   └── edit.rs          # EditOperation type
│   └── error.rs             # Error types and handling
│
├── tests/
│   ├── integration/         # End-to-end file operation tests
│   │   ├── view_tests.rs
│   │   ├── edit_tests.rs
│   │   └── navigation_tests.rs
│   └── unit/                # Unit tests for core logic
│       ├── file_reader_tests.rs
│       ├── line_parser_tests.rs
│       └── backup_tests.rs
│
├── specs/                   # Feature specifications
│   └── 001-gitleaksignore-editor/
│       ├── spec.md          # Requirements specification
│       ├── plan.md          # Implementation plan
│       ├── research.md      # Technical research
│       ├── data-model.md    # Data structures
│       ├── contracts/       # Interface contracts
│       └── quickstart.md    # This file
│
├── Cargo.toml               # Dependencies and project metadata
├── Cargo.lock               # Dependency lock file
├── README.md                # User documentation
└── LICENSE                  # Project license
```

---

## Development Workflow

### TDD (Test-Driven Development)

This project follows Test-Driven Development:

1. **Red**: Write a failing test for the feature
2. **Green**: Write minimal code to make the test pass
3. **Refactor**: Improve code while keeping tests green

### Example TDD Cycle

```bash
# 1. Write a test (in tests/unit/line_parser_tests.rs)
#[test]
fn test_parse_comment_line() {
    let pattern = PatternType::parse("# This is a comment");
    assert_eq!(pattern, PatternType::Comment);
}

# 2. Run the test (it will fail initially)
cargo test test_parse_comment_line

# 3. Implement the feature (in src/core/line_parser.rs)
impl PatternType {
    pub fn parse(line: &str) -> Self {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            return PatternType::Comment;
        }
        // ... other pattern types
    }
}

# 4. Run the test again (it should pass now)
cargo test test_parse_comment_line

# 5. Refactor if needed
```

---

## Building and Running

### Development Build

```bash
# Fast build with debug symbols
cargo build

# Run without building
cargo run -- [ARGS]

# Build and run in one command
cargo run -- --file test.gitleaksignore --lines 10-20
```

### Release Build

```bash
# Optimized build (slower compile, faster runtime)
cargo build --release

# Run release binary
./target/release/gli-editor --help
```

### Watch Mode (Auto-rebuild)

```bash
# Install cargo-watch
cargo install cargo-watch

# Watch for changes and rebuild
cargo watch -x build

# Watch and run tests
cargo watch -x test

# Watch and run specific test
cargo watch -x "test test_pattern_parsing"
```

---

## Running Tests

### All Tests

```bash
# Run all tests (unit + integration)
cargo test

# Verbose output
cargo test -- --nocapture

# Show ignored tests
cargo test -- --ignored
```

### Specific Test Categories

```bash
# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test '*'

# Specific test file
cargo test --test integration_tests

# Specific test function
cargo test test_pattern_parsing
```

### With cargo-nextest (Faster)

```bash
# Install nextest
cargo install cargo-nextest

# Run tests (parallel execution)
cargo nextest run

# Show output for all tests
cargo nextest run --nocapture
```

### Coverage (Optional)

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage

# Open report
open coverage/index.html  # macOS
xdg-open coverage/index.html  # Linux
```

---

## Development Guidelines

### Code Style

- **Format**: Use `rustfmt` (automatically applied on save in most editors)
  ```bash
  cargo fmt
  ```

- **Lint**: Use `clippy` for catching common mistakes
  ```bash
  cargo clippy
  ```

- **Check**: Verify code compiles without warnings
  ```bash
  cargo check --all-targets
  ```

### Commit Message Format

Follow conventional commits:

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types**:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `test`: Adding/updating tests
- `refactor`: Code refactoring
- `perf`: Performance improvement
- `chore`: Maintenance tasks

**Example**:
```
feat(parser): add support for nested archive paths

Implement parsing for gitleaks fingerprints that include nested
archives (e.g., tar.gz!inner.tar!file.env).

Resolves #42
```

### Pull Request Process

1. **Create feature branch**: `git checkout -b feat/your-feature-name`
2. **Write tests first**: Follow TDD approach
3. **Implement feature**: Make tests pass
4. **Run checks**: `cargo fmt && cargo clippy && cargo test`
5. **Commit changes**: Use conventional commit format
6. **Push branch**: `git push origin feat/your-feature-name`
7. **Open PR**: Create pull request on GitHub
8. **Code review**: Address feedback from reviewers
9. **Merge**: Once approved, squash and merge

---

## Debugging

### Using `println!` / `eprintln!`

```rust
// Temporary debugging (remove before commit)
eprintln!("Debug: pattern_type = {:?}", pattern_type);
```

### Using `dbg!` Macro

```rust
// Better than println (shows source location)
let result = dbg!(PatternType::parse(line));
```

### Using rust-lldb / rust-gdb

```bash
# Build with debug symbols
cargo build

# Run with debugger
rust-lldb target/debug/gli-editor
# or
rust-gdb target/debug/gli-editor
```

### Using VSCode

Install Rust Analyzer and CodeLLDB extensions, then use F5 to debug.

---

## Common Tasks

### Add a New Dependency

```bash
# Add to Cargo.toml
cargo add <crate-name>

# Example: Add chrono for timestamps
cargo add chrono

# With specific version
cargo add chrono@0.4

# With features
cargo add tokio --features full
```

### Create a New Module

```bash
# 1. Create file
touch src/core/new_module.rs

# 2. Declare in parent mod.rs
echo "pub mod new_module;" >> src/core/mod.rs

# 3. Implement
cat > src/core/new_module.rs <<EOF
/// Module documentation
pub struct NewThing {
    // fields
}

impl NewThing {
    pub fn new() -> Self {
        Self { /* ... */ }
    }
}
EOF
```

### Run Benchmarks (Future)

```bash
# Create benchmark
mkdir -p benches
cat > benches/parsing_bench.rs <<EOF
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_parsing(c: &mut Criterion) {
    c.bench_function("parse fingerprint", |b| {
        b.iter(|| {
            // benchmark code
        });
    });
}

criterion_group!(benches, benchmark_parsing);
criterion_main!(benches);
EOF

# Run benchmarks
cargo bench
```

---

## Troubleshooting

### Issue: Compilation Error

```
error[E0277]: the trait bound `Foo: Bar` is not satisfied
```

**Solution**: Check that all trait bounds are satisfied, review documentation.

### Issue: Test Failures

```
running 1 test
test test_foo ... FAILED
```

**Solution**:
1. Run with `--nocapture` to see output: `cargo test -- --nocapture`
2. Check test logic and expected values
3. Use `dbg!()` to inspect intermediate values

### Issue: Clippy Warnings

```
warning: unnecessary `unwrap()` call
```

**Solution**: Fix lints before committing:
```bash
cargo clippy --fix
```

### Issue: Slow Compilation

**Solution**:
- Use `cargo check` instead of `cargo build` during development
- Consider using `sccache` for caching
- Reduce debug info in Cargo.toml for dev profile

---

## Performance Profiling

### CPU Profiling

```bash
# Install cargo-flamegraph
cargo install flamegraph

# Generate flamegraph
cargo flamegraph --bin gli-editor -- test.gitleaksignore

# Open flamegraph.svg in browser
```

### Memory Profiling

```bash
# Install valgrind (Linux/macOS)
# Linux: sudo apt install valgrind
# macOS: brew install valgrind

# Run with valgrind
cargo build
valgrind --leak-check=full ./target/debug/gli-editor test.gitleaksignore
```

---

## Resources

### Documentation

- **Rust Book**: https://doc.rust-lang.org/book/
- **Ratatui Docs**: https://ratatui.rs/
- **Crossterm Docs**: https://docs.rs/crossterm
- **tui-textarea**: https://github.com/rhysd/tui-textarea

### Community

- **Project Issues**: https://github.com/epy0n0ff/gli-editor/issues
- **Discussions**: https://github.com/epy0n0ff/gli-editor/discussions
- **Rust Discord**: https://discord.gg/rust-lang

### Specifications

- **Feature Spec**: `specs/001-gitleaksignore-editor/spec.md`
- **Implementation Plan**: `specs/001-gitleaksignore-editor/plan.md`
- **Research**: `specs/001-gitleaksignore-editor/research.md`
- **Data Model**: `specs/001-gitleaksignore-editor/data-model.md`
- **CLI Contract**: `specs/001-gitleaksignore-editor/contracts/cli-interface.md`

---

## Next Steps

1. **Read the Spec**: Start with `specs/001-gitleaksignore-editor/spec.md`
2. **Review the Plan**: Understand the architecture in `plan.md`
3. **Check Research**: See technical decisions in `research.md`
4. **Write Your First Test**: Pick a feature from `spec.md` and write a test
5. **Implement the Feature**: Follow TDD cycle
6. **Submit PR**: Create a pull request when ready

---

## Getting Help

- **Documentation**: Check inline comments and module-level docs
- **Tests**: Look at existing tests for examples
- **Issues**: Search or create a GitHub issue
- **Discussions**: Ask questions in GitHub Discussions

---

**Happy coding!** 🦀
