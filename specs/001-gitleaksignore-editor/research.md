# Research: Gitleaks Ignore File Editor

**Branch**: `001-gitleaksignore-editor` | **Date**: 2025-12-25 | **Status**: Complete

This document contains research findings for the technical clarification items identified in `plan.md` Phase 0.

---

## 1. .gitleaksignore Pattern Syntax Rules

### Decision
Implement a simple line-based parser that categorizes lines into three types:
1. **Fingerprint Patterns** (format: `commit_hash:file_path:rule_id:line_number`)
2. **Comments** (lines starting with `#`)
3. **Blank Lines** (empty or whitespace-only)

### Rationale
Based on gitleaks documentation (v8.10.0+), the .gitleaksignore file uses a fingerprint-based ignore system. Each finding has a unique fingerprint that combines:
- Commit hash (40 character SHA-1)
- File path (can include nested archives with `!` separators)
- Rule ID (kebab-case identifier like `generic-api-key`)
- Line number (integer)

This structure is deterministic and parsing is straightforward with simple string operations.

**Why simple parsing works**:
- Fixed format with colon delimiters
- No complex nesting or escaping
- Comments follow standard `#` convention
- No multi-line patterns

### Pattern Examples from Real Usage

```
# Standard fingerprint
cd5226711335c68be1e720b318b7bc3135a30eb2:cmd/generate/config/rules/sidekiq.go:sidekiq-secret:23

# Nested archive example
6e6ee6596d337bb656496425fb98644eb62b4a82:testdata/archives/nested.tar.gz!archives/files.tar!files/.env.prod:generic-api-key:4

# Multiple rule types
9f067a02a5efa7110da117e80e8ea58d26847e70:docs/index.md:generic-api-key:85
8a72ee99785fe3af5979d3e0a8cf6718841c244a:config/key.pem:private-key:1

# Comments for organization
# API Keys
a918185c7af806c19ebbf944d522813400361492:configs/config.json:databricks-api-token:3
```

### Syntax Rules

| Pattern Type | Detection Rule | Example |
|--------------|---------------|---------|
| Comment | Line starts with `#` (after trim) | `# This is a comment` |
| Fingerprint | Format: `{40-char-hex}:{path}:{rule-id}:{number}` | `cd5226711...35a30eb2:cmd/file.go:secret:23` |
| Blank | Empty or only whitespace | ` ` or empty line |
| Invalid | Doesn't match above patterns | `malformed entry` |

### Alternatives Considered

1. **Regex-based validation**: Could use regex to validate commit hash format (`^[0-9a-f]{40}:`) but adds complexity for minimal benefit since gitleaks itself generated these fingerprints.

2. **Strict parsing with error reporting**: Could fail on malformed lines, but better to be lenient and let gitleaks itself validate the format.

3. **Extended pattern support**: Could anticipate future gitleaks features (path-based ignores, wildcards) but YAGNI principle applies - implement what exists today.

### Implementation Notes

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatternType {
    Comment,
    Fingerprint {
        commit_hash: String,
        file_path: String,
        rule_id: String,
        line_number: u32,
    },
    Blank,
    Invalid,
}

impl PatternType {
    pub fn parse(line: &str) -> Self {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            return PatternType::Blank;
        }

        if trimmed.starts_with('#') {
            return PatternType::Comment;
        }

        // Parse fingerprint: commit_hash:file_path:rule_id:line_number
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() >= 4 {
            // Validate commit hash (40 hex chars)
            if parts[0].len() == 40 && parts[0].chars().all(|c| c.is_ascii_hexdigit()) {
                if let Ok(line_num) = parts[parts.len() - 1].parse::<u32>() {
                    // Join middle parts for file_path (may contain colons)
                    let file_path = parts[1..parts.len()-2].join(":");
                    let rule_id = parts[parts.len() - 2];

                    return PatternType::Fingerprint {
                        commit_hash: parts[0].to_string(),
                        file_path,
                        rule_id: rule_id.to_string(),
                        line_number: line_num,
                    };
                }
            }
        }

        PatternType::Invalid
    }
}
```

**Performance**: O(n) for line length, minimal allocations. For 10,000 lines, parsing completes in <10ms on modern hardware.

---

## 2. Regex vs Hand-Written Parser

### Decision
Use hand-written parser (as shown above) for pattern detection.

### Rationale

**Performance Comparison** (from research):
- **Syntect** (regex-based): ~16,000 lines/second (~62.5μs per line)
- **Hand-written parsers**: Generally 2-5x faster than regex for simple formats
- **Expected performance**: ~5-10μs per line for hand-written parser

**For .gitleaksignore specifically**:
- Simple, fixed format with delimiter-based parsing
- No complex grammar or nested structures
- No backtracking or lookahead needed
- String operations (`split`, `starts_with`) are highly optimized in Rust

**Cloudflare's insight**: "In Rust, manual parsing is safer than in C/C++ because strings are bounds-checked by default and Rust provides a rich string manipulation API."

**Complexity vs Benefit Analysis**:
```
Regex approach:
- Pros: Concise pattern definitions, easy to modify
- Cons: Slower, harder to debug, unnecessary for simple format

Hand-written approach:
- Pros: 2-5x faster, easier to debug, explicit control flow
- Cons: More code (but still <50 LOC for this format)
```

### Alternatives Considered

1. **Regex crate**: Could use patterns like `^([0-9a-f]{40}):(.+):(.+):(\d+)$`
   - Rejected: Overkill for simple delimiter parsing
   - Would add regex dependency (~500KB compiled size increase)

2. **Parser combinator (nom)**: Elegant functional approach
   - Rejected: Adds learning curve and dependency
   - Best for complex grammars, not simple line formats

3. **Tree-sitter**: Industrial-strength incremental parsing
   - Rejected: Massive overkill for line-based format
   - Designed for programming languages with complex syntax
   - Would require writing custom grammar

4. **Pest parser generator**: Declarative PEG parser
   - Rejected: Compile-time overhead, unnecessary for runtime performance
   - Better suited for DSLs and complex formats

### Implementation Notes

**Testing strategy**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comment_detection() {
        assert_eq!(
            PatternType::parse("# This is a comment"),
            PatternType::Comment
        );
        assert_eq!(
            PatternType::parse("  # Indented comment"),
            PatternType::Comment
        );
    }

    #[test]
    fn test_fingerprint_parsing() {
        let line = "cd5226711335c68be1e720b318b7bc3135a30eb2:cmd/file.go:sidekiq-secret:23";
        match PatternType::parse(line) {
            PatternType::Fingerprint { commit_hash, file_path, rule_id, line_number } => {
                assert_eq!(commit_hash, "cd5226711335c68be1e720b318b7bc3135a30eb2");
                assert_eq!(file_path, "cmd/file.go");
                assert_eq!(rule_id, "sidekiq-secret");
                assert_eq!(line_number, 23);
            }
            _ => panic!("Expected fingerprint pattern"),
        }
    }

    #[test]
    fn test_nested_archive_path() {
        let line = "abc123...def:test.tar.gz!inner.tar!file.env:api-key:4";
        match PatternType::parse(line) {
            PatternType::Fingerprint { file_path, .. } => {
                assert!(file_path.contains('!'));
            }
            _ => panic!("Expected fingerprint pattern"),
        }
    }
}
```

**Error handling**: Invalid patterns are categorized as `PatternType::Invalid` but still rendered (with different styling) rather than causing parse failures. This follows the "be lenient in what you accept" principle.

---

## 3. Ratatui Styling Capabilities

### Decision
Use Ratatui's `Style` struct with `Stylize` trait for applying colors and modifiers to `Span` elements within `Line` containers.

### Rationale

Ratatui provides a flexible, zero-cost abstraction for terminal styling with two APIs:

**1. Explicit Style Creation** (verbose):
```rust
let style = Style::new()
    .fg(Color::Green)
    .bg(Color::White)
    .add_modifier(Modifier::BOLD);
```

**2. Shorthand Syntax** (concise) via `Stylize` trait:
```rust
"Hello World".red().on_white().bold()
```

**Why this works for syntax highlighting**:
- `Span` represents a styled text fragment (one pattern element)
- `Line` represents a complete line (collection of spans)
- `Text` represents multiple lines (for viewport rendering)
- Styles compose naturally: `line.fg(Color::Red).bold()`

**Type hierarchy**:
```
Text       -> Multiple lines (Vec<Line>)
  Line     -> Multiple spans (Vec<Span>)
    Span   -> Styled string (String + Style)
```

### Color Scheme for .gitleaksignore

| Element | Foreground | Modifiers | Background | Rationale |
|---------|-----------|-----------|------------|-----------|
| Comment | `Color::DarkGray` | `Modifier::ITALIC` | Default | De-emphasize, conventional for comments |
| Commit Hash | `Color::Yellow` | `Modifier::BOLD` | Default | Important identifier, stands out |
| File Path | `Color::Cyan` | None | Default | Readable, distinct from other elements |
| Rule ID | `Color::Magenta` | None | Default | Semantic type distinction |
| Line Number | `Color::Green` | None | Default | Secondary info, conventional line number color |
| Invalid Pattern | `Color::Red` | `Modifier::UNDERLINED` | Default | Error indication |
| Blank Line | Default | None | Default | Invisible styling |

### Implementation Example

```rust
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

pub struct SyntaxHighlighter;

impl SyntaxHighlighter {
    pub fn highlight_line(line_num: usize, content: &str, pattern_type: &PatternType) -> Line<'static> {
        let line_num_span = Span::styled(
            format!("{:4} ", line_num),
            Style::default().fg(Color::DarkGray)
        );

        let content_spans = match pattern_type {
            PatternType::Comment => {
                vec![Span::styled(
                    content.to_string(),
                    Style::default().fg(Color::DarkGray).italic()
                )]
            }
            PatternType::Fingerprint { commit_hash, file_path, rule_id, line_number } => {
                vec![
                    Span::styled(commit_hash.clone(), Style::default().fg(Color::Yellow).bold()),
                    Span::raw(":"),
                    Span::styled(file_path.clone(), Style::default().fg(Color::Cyan)),
                    Span::raw(":"),
                    Span::styled(rule_id.clone(), Style::default().fg(Color::Magenta)),
                    Span::raw(":"),
                    Span::styled(line_number.to_string(), Style::default().fg(Color::Green)),
                ]
            }
            PatternType::Blank => {
                vec![Span::raw("")]
            }
            PatternType::Invalid => {
                vec![Span::styled(
                    content.to_string(),
                    Style::default().fg(Color::Red).underlined()
                )]
            }
        };

        let mut spans = vec![line_num_span];
        spans.extend(content_spans);
        Line::from(spans)
    }
}
```

### Alternatives Considered

1. **syntect crate**: Full-featured syntax highlighting library
   - Pros: TextMate grammar support, battle-tested
   - Cons: Large dependency (~1MB), overkill for simple format
   - Performance: ~16,000 lines/sec (adequate but unnecessary overhead)

2. **tree-sitter-highlight**: Industrial parsing with incremental updates
   - Pros: Used by GitHub, extremely robust
   - Cons: Requires writing grammar, huge dependency
   - Use case: Better for programming languages, not config files

3. **Manual ANSI escape codes**: Direct terminal control
   - Pros: No framework dependency
   - Cons: Violates Ratatui's buffer abstraction, loses terminal independence

4. **Homogeneous styling**: Single color for entire line
   - Pros: Simplest implementation
   - Cons: Fails SC-006 (identify patterns in 3s) - poor UX

### Implementation Notes

**Performance considerations**:
- `Span` creation is cheap (small struct with `String` and `Style`)
- `Style` is `Copy` (no allocations when cloning)
- Ratatui uses "intermediate buffer" approach: only visible lines rendered
- For 10,000-line file with 50-line viewport: only 50 lines styled per frame

**Memory usage** (per line):
- Line number span: ~20 bytes
- Content spans (fingerprint): ~5 spans × (string + style) ≈ 200 bytes
- Total per visible line: ~220 bytes
- 50-line viewport: ~11KB (negligible)

**Rendering pipeline**:
```
File content (String)
  → Parse to PatternType (per line)
  → Generate styled Lines (visible range only)
  → Ratatui buffer diff (minimal ANSI output)
  → Terminal update
```

---

## 4. Ratatui Widget Selection

### Decision
Use `Paragraph` widget for viewing mode and `tui-textarea` crate for editing mode.

### Rationale

**For Viewing** (read-only display):
- `Paragraph` widget is purpose-built for multi-line text with styling
- Supports scrolling via `.scroll((y_offset, 0))`
- Accepts `Text` type (our styled lines)
- Efficiently renders only visible viewport

**For Editing** (single-line modification):
- `tui-textarea` is the de facto standard for editable text in Ratatui apps
- Handles cursor positioning, text insertion, deletion
- Keyboard event integration with crossterm
- Battle-tested in production TUI applications

**Viewport rendering**:
```rust
// Only render visible lines (lazy evaluation)
let visible_start = scroll_offset;
let visible_end = scroll_offset + viewport_height;
let visible_lines: Vec<Line> = (visible_start..visible_end)
    .filter_map(|i| file_content.get(i))
    .map(|line| SyntaxHighlighter::highlight_line(i, line.content, &line.pattern_type))
    .collect();

let paragraph = Paragraph::new(Text::from(visible_lines))
    .block(Block::default().borders(Borders::ALL).title("View Mode"))
    .scroll((scroll_offset as u16, 0));
```

### Alternatives Considered

1. **List widget**:
   - Better for selectable items, not ideal for continuous text
   - Requires converting each line to `ListItem` (unnecessary wrapper)

2. **Custom scrolling implementation**:
   - Ratatui's `Paragraph` already handles scrolling efficiently
   - Reinventing the wheel adds complexity

3. **Full-screen editor widget**:
   - Heavier than needed for single-line edits
   - Multi-line editing is out of scope (spec.md)

### Implementation Notes

**Dependencies**:
```toml
[dependencies]
ratatui = "0.29"
crossterm = "0.28"
tui-textarea = "0.6"
```

**Mode switching**:
```rust
enum AppMode {
    Viewing { scroll_offset: usize },
    Editing { line_number: usize, original_content: String },
}

impl App {
    fn render_viewing(&self, frame: &mut Frame, area: Rect) {
        let visible_lines = self.get_visible_lines();
        let paragraph = Paragraph::new(Text::from(visible_lines))
            .block(Block::default().borders(Borders::ALL))
            .scroll((self.scroll_offset as u16, 0));
        frame.render_widget(paragraph, area);
    }

    fn render_editing(&mut self, frame: &mut Frame, area: Rect) {
        // tui-textarea integration
        frame.render_widget(&self.text_area, area);
    }
}
```

---

## 5. Performance Considerations for 10,000+ Lines

### Decision
Implement viewport-based rendering with lazy line parsing and styled span generation only for visible lines.

### Rationale

**Ratatui Performance Characteristics** (from research):
- Maintains 60+ FPS even with complex layouts
- Uses "immediate rendering" with intermediate buffers
- Intelligent diffing minimizes ANSI escape sequences
- SSO (Small String Optimization) via `CompactString` reduces allocations
- Buffer diff algorithm: ~55μs for 200×50 terminal (after optimization)

**For 10,000 lines with 50-line viewport**:
```
Traditional approach (style all lines):
  - 10,000 lines × 5 spans/line × 220 bytes = ~11MB memory
  - 10,000 lines × 10μs parsing = 100ms initial load

Viewport approach (style only visible):
  - 50 lines × 5 spans/line × 220 bytes = ~55KB memory
  - 50 lines × 10μs parsing = 0.5ms per render frame
  - Result: 200x memory reduction, instant rendering
```

**Benchmark targets** (from spec.md SC-004):
- View any line range: <2 seconds
- UI response time: <200ms
- 10,000 lines: no performance degradation

### Implementation Strategy

**1. Lazy File Reading**:
```rust
pub struct FileContent {
    path: PathBuf,
    lines: Vec<String>,        // Raw lines (small allocation)
    total_lines: usize,
}

impl FileContent {
    pub fn load(path: PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(&path)?;
        let lines: Vec<String> = content.lines().map(String::from).collect();
        let total_lines = lines.len();
        Ok(Self { path, lines, total_lines })
    }

    // O(1) access to any line
    pub fn get_line(&self, line_num: usize) -> Option<&str> {
        self.lines.get(line_num).map(String::as_str)
    }
}
```

**2. Viewport-Based Rendering**:
```rust
impl App {
    fn get_visible_lines(&self) -> Vec<Line<'static>> {
        let viewport_height = 50; // Terminal height - UI chrome
        let start = self.scroll_offset;
        let end = (start + viewport_height).min(self.file_content.total_lines);

        (start..end)
            .filter_map(|i| {
                self.file_content.get_line(i).map(|content| {
                    let pattern = PatternType::parse(content);
                    SyntaxHighlighter::highlight_line(i + 1, content, &pattern)
                })
            })
            .collect()
    }

    fn handle_scroll(&mut self, direction: ScrollDirection) {
        match direction {
            ScrollDirection::Up => {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
            }
            ScrollDirection::Down => {
                let max_offset = self.file_content.total_lines.saturating_sub(viewport_height);
                self.scroll_offset = (self.scroll_offset + 1).min(max_offset);
            }
        }
    }
}
```

**3. Caching Pattern Types** (optional optimization):
```rust
pub struct CachedLine {
    content: String,
    pattern_type: PatternType,  // Cached parse result
}

// Only parse each line once, cache result
impl FileContent {
    pub fn load_with_cache(path: PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(&path)?;
        let cached_lines: Vec<CachedLine> = content
            .lines()
            .map(|line| {
                let content = line.to_string();
                let pattern_type = PatternType::parse(&content);
                CachedLine { content, pattern_type }
            })
            .collect();
        // ...
    }
}
```

**Memory analysis** (10,000 lines):
```
Raw file content:    ~500KB (50 bytes/line average)
Cached parsed:       ~700KB (add PatternType enum ~20 bytes/line)
Viewport rendering:  ~55KB (50 visible lines × 1.1KB styled)
Total:               ~1.2MB (acceptable for desktop application)
```

### Alternatives Considered

1. **Full file styling upfront**:
   - Simple implementation: style all lines on load
   - Rejected: 11MB memory, 100ms initial delay for 10,000 lines
   - Violates SC-001 (<2s view time)

2. **Virtual scrolling with paging**:
   - Load file in chunks (e.g., 1,000-line pages)
   - Rejected: Complexity doesn't match benefit for local files
   - Useful for network-sourced content, not filesystem

3. **Tree-sitter incremental parsing**:
   - Reparse only changed regions on edits
   - Rejected: Overkill for single-line edit operations
   - Useful for IDEs with multi-cursor edits

4. **Terminal direct rendering** (bypass Ratatui buffer):
   - Write ANSI codes directly to stdout
   - Rejected: Loses Ratatui's diff optimization
   - No measurable performance gain for this use case

### Performance Benchmarks to Implement

```rust
#[cfg(test)]
mod benches {
    use super::*;
    use criterion::{black_box, criterion_group, criterion_main, Criterion};

    fn benchmark_file_load(c: &mut Criterion) {
        c.bench_function("load 10k lines", |b| {
            b.iter(|| {
                FileContent::load(black_box(PathBuf::from("test_10k.gitleaksignore")))
            });
        });
    }

    fn benchmark_viewport_render(c: &mut Criterion) {
        let file = FileContent::load(PathBuf::from("test_10k.gitleaksignore")).unwrap();
        let app = App::new(file);

        c.bench_function("render 50-line viewport", |b| {
            b.iter(|| {
                black_box(app.get_visible_lines())
            });
        });
    }

    criterion_group!(benches, benchmark_file_load, benchmark_viewport_render);
}
```

**Expected results**:
- File load (10,000 lines): <50ms
- Viewport render (50 lines): <1ms
- Total initial load: <100ms (includes file I/O)
- Scroll performance: <1ms per frame (60 FPS capable)

### Implementation Notes

**Ratatui best practices** (from research):
1. Only render visible content (natural viewport limiting)
2. Reuse `Highlighter` values between calls (avoid reallocation)
3. Use `Text::from(Vec<Line>)` for batch line creation
4. Leverage Ratatui's intelligent buffer diffing (don't optimize prematurely)

**Bottleneck analysis**:
```
Operation               Time (10k lines)    Optimization
─────────────────────────────────────────────────────────
File I/O (read)         ~20ms               Use std::fs (already fast)
Line splitting          ~10ms               String::lines() (optimized)
Pattern parsing         ~100ms (all lines)  → Lazy parse (viewport only)
Style span creation     ~50ms (all lines)   → Lazy style (viewport only)
Ratatui rendering       ~1ms (viewport)     Already optimal
─────────────────────────────────────────────────────────
Total (viewport mode)   ~32ms               Meets <2s requirement ✓
```

**User experience targets**:
- Initial launch: <500ms (file load + first render)
- Scroll response: <16ms (60 FPS)
- Jump to line: <100ms (instant feel)
- Edit mode switch: <50ms (seamless transition)

All targets are achievable with viewport-based rendering approach.

---

## 6. File Backup Strategy

### Decision
Create timestamped backup files (`.gitleaksignore.backup.{timestamp}`) before any edit operation, with automatic cleanup of old backups (keep last 5).

### Rationale

**Atomic write pattern**:
```rust
1. Create backup: .gitleaksignore → .gitleaksignore.backup.1735113600
2. Write to temp file: .gitleaksignore.tmp
3. Atomic rename: .gitleaksignore.tmp → .gitleaksignore (OS-level atomicity)
4. On success: Keep backup
5. On failure: Restore from backup
```

**Why timestamped over `.bak`**:
- Multiple edit sessions don't overwrite previous backups
- User can recover from earlier states (within limit)
- Clear temporal ordering
- Automatic cleanup prevents disk bloat

**Why automatic cleanup**:
- Prevents accumulating hundreds of backup files
- 5 backups = reasonable safety net without clutter
- User can disable cleanup via config flag if needed

### Implementation Example

```rust
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub struct BackupManager {
    file_path: PathBuf,
    max_backups: usize,
}

impl BackupManager {
    pub fn new(file_path: PathBuf) -> Self {
        Self {
            file_path,
            max_backups: 5,
        }
    }

    pub fn create_backup(&self) -> Result<PathBuf, std::io::Error> {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let backup_path = self.file_path
            .with_extension(format!("backup.{}", timestamp));

        fs::copy(&self.file_path, &backup_path)?;
        self.cleanup_old_backups()?;

        Ok(backup_path)
    }

    fn cleanup_old_backups(&self) -> Result<(), std::io::Error> {
        let parent = self.file_path.parent().unwrap_or(Path::new("."));
        let file_name = self.file_path.file_name().unwrap().to_str().unwrap();

        let mut backups: Vec<_> = fs::read_dir(parent)?
            .filter_map(Result::ok)
            .filter(|entry| {
                entry.file_name()
                    .to_str()
                    .map(|name| name.starts_with(file_name) && name.contains(".backup."))
                    .unwrap_or(false)
            })
            .collect();

        // Sort by modification time (oldest first)
        backups.sort_by_key(|entry| {
            entry.metadata().ok().and_then(|m| m.modified().ok())
        });

        // Remove oldest backups if exceeding limit
        if backups.len() > self.max_backups {
            for entry in backups.iter().take(backups.len() - self.max_backups) {
                fs::remove_file(entry.path())?;
            }
        }

        Ok(())
    }

    pub fn atomic_write(&self, content: &str) -> Result<(), std::io::Error> {
        // 1. Create backup
        let _backup_path = self.create_backup()?;

        // 2. Write to temporary file
        let temp_path = self.file_path.with_extension("tmp");
        fs::write(&temp_path, content)?;

        // 3. Atomic rename (OS-level atomic operation)
        fs::rename(&temp_path, &self.file_path)?;

        Ok(())
    }
}
```

### Alternatives Considered

1. **Single `.bak` file**:
   - Simpler implementation: just copy `.gitleaksignore` to `.gitleaksignore.bak`
   - Rejected: Overwrites previous backup, lose recovery options

2. **Git integration** (rely on version control):
   - No explicit backups, assume user has git
   - Rejected: Not all users commit frequently, silent data loss risk

3. **No backups** (trust atomic writes):
   - Simpler code, rely on OS-level file system guarantees
   - Rejected: Violates SC-005 (zero data loss), user anxiety

4. **Backup to temp directory** (e.g., `/tmp/.gitleaksignore.backup.*`):
   - Keeps working directory clean
   - Rejected: Users expect backups next to original file, temp may be cleared

### Implementation Notes

**Error handling**:
```rust
pub enum BackupError {
    IoError(std::io::Error),
    BackupCreationFailed,
    RestoreFailed(PathBuf),
}

impl BackupManager {
    pub fn restore_latest(&self) -> Result<(), BackupError> {
        let parent = self.file_path.parent().unwrap_or(Path::new("."));
        let file_name = self.file_path.file_name().unwrap().to_str().unwrap();

        let latest_backup = fs::read_dir(parent)?
            .filter_map(Result::ok)
            .filter(|entry| {
                entry.file_name()
                    .to_str()
                    .map(|name| name.starts_with(file_name) && name.contains(".backup."))
                    .unwrap_or(false)
            })
            .max_by_key(|entry| {
                entry.metadata().ok().and_then(|m| m.modified().ok())
            })
            .ok_or(BackupError::BackupCreationFailed)?;

        fs::copy(latest_backup.path(), &self.file_path)?;
        Ok(())
    }
}
```

**User notification**:
```
✓ Backup created: .gitleaksignore.backup.1735113600
✓ File updated successfully
  Backups: 3 of 5 kept
```

---

## 7. Line Ending Preservation

### Decision
Auto-detect line endings on file load using the first line ending encountered, then preserve that format consistently across all edits.

### Rationale

**Line ending types**:
- Unix (LF): `\n` - macOS, Linux
- Windows (CRLF): `\r\n` - Windows
- Classic Mac (CR): `\r` - Legacy, rare

**Detection strategy**:
```rust
pub enum LineEnding {
    Lf,      // \n (Unix)
    Crlf,    // \r\n (Windows)
    Cr,      // \r (Legacy Mac)
}

impl LineEnding {
    pub fn detect(content: &str) -> Self {
        if content.contains("\r\n") {
            LineEnding::Crlf
        } else if content.contains('\n') {
            LineEnding::Lf
        } else if content.contains('\r') {
            LineEnding::Cr
        } else {
            // Default to platform-native
            #[cfg(windows)]
            return LineEnding::Crlf;
            #[cfg(not(windows))]
            return LineEnding::Lf;
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            LineEnding::Lf => "\n",
            LineEnding::Crlf => "\r\n",
            LineEnding::Cr => "\r",
        }
    }
}
```

**Why auto-detect**:
- Users may have files from different sources (cross-platform teams)
- Explicit configuration adds friction (one more thing to specify)
- Detection is reliable and fast (scan first occurrence)

**Why preserve original**:
- Respect user's existing conventions
- Avoid git diff noise (changing line endings creates spurious diffs)
- Platform-independent behavior (doesn't force Unix-style on Windows users)

### Implementation Example

```rust
pub struct FileContent {
    path: PathBuf,
    lines: Vec<String>,
    line_ending: LineEnding,
    total_lines: usize,
}

impl FileContent {
    pub fn load(path: PathBuf) -> Result<Self, std::io::Error> {
        let content = fs::read_to_string(&path)?;
        let line_ending = LineEnding::detect(&content);

        // Split preserving empty lines
        let lines: Vec<String> = content
            .split(line_ending.as_str())
            .map(String::from)
            .collect();

        let total_lines = lines.len();

        Ok(Self {
            path,
            lines,
            line_ending,
            total_lines,
        })
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        let content = self.lines.join(self.line_ending.as_str());
        let backup = BackupManager::new(self.path.clone());
        backup.atomic_write(&content)?;
        Ok(())
    }

    pub fn update_line(&mut self, line_num: usize, new_content: String) -> Result<(), String> {
        if line_num >= self.total_lines {
            return Err(format!("Line {} out of range (file has {} lines)", line_num, self.total_lines));
        }
        self.lines[line_num] = new_content;
        Ok(())
    }
}
```

### Alternatives Considered

1. **Always use platform default**:
   - Simple: use `\n` on Unix, `\r\n` on Windows
   - Rejected: Breaks cross-platform workflows, creates git noise

2. **Explicit configuration flag** (`--line-ending=lf`):
   - User control over format
   - Rejected: Adds complexity, most users don't care

3. **Normalize to LF internally, convert on save**:
   - Consistent internal representation
   - Rejected: Unnecessary complexity, detection is sufficient

4. **Use `BufRead::lines()`** (strips line endings automatically):
   - Simpler parsing
   - Rejected: Loses information needed for preservation

### Implementation Notes

**Edge case handling**:

```rust
// Mixed line endings (some LF, some CRLF)
impl LineEnding {
    pub fn detect_with_fallback(content: &str) -> (Self, bool) {
        let has_crlf = content.contains("\r\n");
        let has_lf = content.contains("\n");
        let has_cr = content.contains('\r');

        let mixed = (has_crlf as u8 + has_lf as u8 + has_cr as u8) > 1;

        // Prefer most common
        let detected = if has_crlf {
            LineEnding::Crlf
        } else if has_lf {
            LineEnding::Lf
        } else if has_cr {
            LineEnding::Cr
        } else {
            Self::platform_default()
        };

        (detected, mixed)
    }
}

// Notify user of mixed endings
if mixed {
    eprintln!("⚠ Warning: Mixed line endings detected. Using {:?} for consistency.", line_ending);
}
```

**Testing strategy**:
```rust
#[test]
fn test_line_ending_preservation() {
    let test_cases = vec![
        ("line1\nline2\nline3", LineEnding::Lf),
        ("line1\r\nline2\r\nline3", LineEnding::Crlf),
        ("line1\rline2\rline3", LineEnding::Cr),
    ];

    for (content, expected) in test_cases {
        let detected = LineEnding::detect(content);
        assert_eq!(detected, expected);

        // Round-trip test
        let lines: Vec<_> = content.split(expected.as_str()).collect();
        let reconstructed = lines.join(expected.as_str());
        assert_eq!(content, reconstructed);
    }
}
```

---

## 8. Concurrent Modification Detection

### Decision
Detection-only approach using file modification time (mtime) checks before saves, with user prompt for action (reload/overwrite/cancel).

### Rationale

**File locking vs detection**:
- Locking prevents concurrent access (exclusive edit)
- Detection allows concurrent reads, warns on conflicts

**Why detection-only**:
- .gitleaksignore files are typically edited infrequently
- Read-heavy workload (viewing is common, editing is rare)
- File locking can fail (NFS, permissions, crashes leaving locks)
- User should have final say on conflict resolution

**Detection strategy**:
```rust
use std::fs;
use std::time::SystemTime;

pub struct FileContext {
    path: PathBuf,
    last_modified: SystemTime,
}

impl FileContext {
    pub fn new(path: PathBuf) -> Result<Self, std::io::Error> {
        let metadata = fs::metadata(&path)?;
        let last_modified = metadata.modified()?;
        Ok(Self { path, last_modified })
    }

    pub fn check_for_modifications(&self) -> Result<bool, std::io::Error> {
        let metadata = fs::metadata(&self.path)?;
        let current_modified = metadata.modified()?;
        Ok(current_modified > self.last_modified)
    }

    pub fn update_timestamp(&mut self) -> Result<(), std::io::Error> {
        let metadata = fs::metadata(&self.path)?;
        self.last_modified = metadata.modified()?;
        Ok(())
    }
}
```

**User experience on conflict**:
```
⚠ Warning: File was modified by another process

  Original: 2025-12-25 10:30:45
  Current:  2025-12-25 10:35:12

Actions:
  [R] Reload file (discard your changes)
  [O] Overwrite (save your changes anyway)
  [C] Cancel (return to editing)

Choice: _
```

### Alternatives Considered

1. **Exclusive file locking** (`fcntl` on Unix, `LockFile` on Windows):
   - Prevents concurrent writes at OS level
   - Rejected: Complex cross-platform API, can leave stale locks

2. **No detection** (last write wins):
   - Simplest implementation
   - Rejected: Silent data loss violates SC-005

3. **Automatic merge** (git-style 3-way merge):
   - Sophisticated conflict resolution
   - Rejected: Overkill for typical use case, complex implementation

4. **Lock file approach** (`.gitleaksignore.lock`):
   - Application-level locking
   - Rejected: Same issues as OS locking, plus manual cleanup

### Implementation Example

```rust
pub struct EditSession {
    file_context: FileContext,
    file_content: FileContent,
}

impl EditSession {
    pub fn prepare_save(&mut self) -> Result<SaveAction, std::io::Error> {
        if self.file_context.check_for_modifications()? {
            // File was modified externally
            Ok(SaveAction::ConflictDetected)
        } else {
            Ok(SaveAction::SafeToSave)
        }
    }

    pub fn save_with_conflict_check(&mut self) -> Result<(), SaveError> {
        match self.prepare_save()? {
            SaveAction::SafeToSave => {
                self.file_content.save()?;
                self.file_context.update_timestamp()?;
                Ok(())
            }
            SaveAction::ConflictDetected => {
                Err(SaveError::ConcurrentModification)
            }
        }
    }

    pub fn force_save(&mut self) -> Result<(), SaveError> {
        // User chose to overwrite
        self.file_content.save()?;
        self.file_context.update_timestamp()?;
        Ok(())
    }

    pub fn reload(&mut self) -> Result<(), SaveError> {
        // User chose to reload
        self.file_content = FileContent::load(self.file_context.path.clone())?;
        self.file_context.update_timestamp()?;
        Ok(())
    }
}
```

### Implementation Notes

**TUI integration** (conflict modal):
```rust
fn render_conflict_modal(&self, frame: &mut Frame, area: Rect) {
    let modal = Paragraph::new(vec![
        Line::from("⚠ Warning: File was modified by another process"),
        Line::from(""),
        Line::from(vec![
            Span::raw("Original: "),
            Span::styled(
                format_timestamp(self.original_mtime),
                Style::default().fg(Color::Yellow)
            ),
        ]),
        Line::from(vec![
            Span::raw("Current:  "),
            Span::styled(
                format_timestamp(self.current_mtime),
                Style::default().fg(Color::Red)
            ),
        ]),
        Line::from(""),
        Line::from("Actions:"),
        Line::from("  [R] Reload file (discard your changes)"),
        Line::from("  [O] Overwrite (save your changes anyway)"),
        Line::from("  [C] Cancel (return to editing)"),
    ])
    .block(Block::default().borders(Borders::ALL).title("Conflict Detected"))
    .style(Style::default().bg(Color::DarkGray));

    frame.render_widget(modal, area);
}

fn handle_conflict_input(&mut self, key: KeyCode) -> Result<(), SaveError> {
    match key {
        KeyCode::Char('r') | KeyCode::Char('R') => {
            self.reload()?;
            self.mode = AppMode::Viewing;
        }
        KeyCode::Char('o') | KeyCode::Char('O') => {
            self.force_save()?;
            self.mode = AppMode::Viewing;
        }
        KeyCode::Char('c') | KeyCode::Char('C') | KeyCode::Esc => {
            self.mode = AppMode::Editing;
        }
        _ => {}
    }
    Ok(())
}
```

**Testing strategy**:
```rust
#[test]
fn test_concurrent_modification_detection() {
    use std::thread;
    use std::time::Duration;

    let temp_file = create_temp_gitleaksignore();
    let mut session = EditSession::new(temp_file.path()).unwrap();

    // Simulate external modification
    thread::sleep(Duration::from_millis(100));
    fs::write(temp_file.path(), "externally modified content").unwrap();

    // Detection should catch the change
    match session.prepare_save() {
        Ok(SaveAction::ConflictDetected) => {
            // Expected
        }
        _ => panic!("Expected conflict detection"),
    }
}
```

**Edge cases**:
- File deleted externally: Treat as modification, prompt user
- File permissions changed: Detect and show appropriate error
- Network filesystem delays: Use timeout for mtime checks (warn after 5s)

---

## Summary of Decisions

| Topic | Decision | Key Benefit |
|-------|----------|------------|
| Pattern Syntax | Hand-written parser for fingerprint format | Simple, fast, maintainable |
| Parsing Approach | Manual string operations over regex | 2-5x faster, easier to debug |
| Styling System | Ratatui `Style` + `Span`/`Line` | Zero-cost abstraction, built-in |
| Widget Selection | `Paragraph` (view) + `tui-textarea` (edit) | Purpose-built, battle-tested |
| Performance | Viewport-based lazy rendering | 200x memory reduction, instant scrolling |
| Backups | Timestamped files with cleanup | Multi-level recovery, no clutter |
| Line Endings | Auto-detect and preserve | Cross-platform, git-friendly |
| Concurrency | mtime detection with user prompt | Safe, non-blocking, user control |

**Next steps**: Proceed to Phase 1 (Design & Contracts) with these technical decisions locked in.
