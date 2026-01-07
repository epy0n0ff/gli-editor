# gli-editor

Terminal editor for `.gitleaksignore` files with syntax highlighting and inline editing.

## Features

- 📄 **View Mode**: Browse `.gitleaksignore` files with syntax highlighting
- 👁️ **Live Preview Pane**: Automatically displays the source file and highlighted line referenced by fingerprints
- 👉 **Current Line Indicator**: Visual cursor showing active line (yellow line number + background highlight)
- ✏️ **Edit Mode**: Inline editing of individual entries
- 🗑️ **Delete Lines**: Remove entries with vim-style `dd` command or Delete key
- 🎨 **Syntax Highlighting**: Pattern-based coloring for fingerprints, comments, and invalid entries
- 🔒 **Safe Editing**: Automatic backups and atomic file writes
- ⚡ **Fast**: Optimized for files with 10,000+ lines
- 🎯 **Precise Navigation**: Jump to specific lines or ranges

## Installation

```bash
cargo install --path .
```

## Usage

```bash
# Open default .gitleaksignore in current directory
gli-editor

# Open specific file
gli-editor --file /path/to/.gitleaksignore

# View specific line with context (default: ±3 lines)
gli-editor --lines 42

# View line with custom context
gli-editor --lines 42 --context 5
# Or use compact notation
gli-editor --lines 42+5

# View line range
gli-editor --lines 10-50

# Read-only mode (disable editing)
gli-editor --read-only

# Show help
gli-editor --help
```

## CLI Options

- `-f, --file <PATH>` - Path to .gitleaksignore file (default: `./.gitleaksignore`)
- `-l, --lines <SPEC>` - Line specification: `42` (single line), `10-50` (range), `42+5` (line 42 with ±5 context)
- `-C, --context <NUM>` - Number of context lines around target line (default: 3)
- `-r, --read-only` - Launch in read-only mode (disable editing)
- `-h, --help` - Print help information
- `-V, --version` - Print version information

## Keybindings

### View Mode
- `j` / `↓` - Scroll down
- `k` / `↑` - Scroll up
- `d` / `PageDown` - Page down
- `u` / `PageUp` - Page up
- `g` / `Home` - Jump to top
- `G` / `End` - Jump to bottom
- `p` - Toggle preview pane on/off
- `i` / `Enter` - Edit current line
- `dd` / `Delete` - Delete current line (creates backup)
- `q` - Quit

### Edit Mode
- `Esc` - Save and exit edit mode
- `Ctrl+C` - Cancel edit (discard changes)
- `←` / `→` - Move cursor
- `Home` / `End` - Jump to line start/end
- `Backspace` / `Delete` - Delete characters

## Syntax Highlighting

gli-editor provides visual distinction for different pattern types:

- **Comments** (lines starting with `#`) - Dark gray, italic
- **Fingerprints** - Color-coded components:
  - Commit hash (40 hex chars) - **Yellow + Bold**
  - File path - **Cyan**
  - Rule ID - **Magenta**
  - Line number - **Green**
- **Invalid patterns** - **Red + Underlined**
- **Blank lines** - Default styling

Example fingerprint:
```
cd5226711335c68be1e720b318b7bc3135a30eb2:cmd/file.go:sidekiq-secret:23
 └─────────────────────────────────────────┘ └──────────┘ └────────────┘ └┘
          Yellow (Commit Hash)                  Cyan         Magenta    Green
                                              (File Path)   (Rule ID)  (Line#)
```

## Safety Features

- **Automatic Backups**: Every edit creates a timestamped backup (`.gitleaksignore.backup.{timestamp}`)
- **Backup Management**: Automatically keeps the last 5 backups, removes older ones
- **Atomic Writes**: File writes use temporary files and atomic rename to prevent corruption
- **Concurrent Modification Detection**: Warns if file was modified externally during editing
- **Line Ending Preservation**: Automatically detects and preserves LF, CRLF, or CR line endings

## Development

```bash
# Build
cargo build

# Run tests
cargo test

# Run with arguments
cargo run -- --help

# Format code
cargo fmt

# Lint
cargo clippy
```

## License

MIT

## Repository

https://github.com/epy0n0ff/gli-editor
