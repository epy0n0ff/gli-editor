# CLI Interface Contract: Gitleaks Ignore File Editor

**Feature**: 001-gitleaksignore-editor
**Date**: 2025-12-25
**Version**: 1.0.0

## Purpose

This document specifies the command-line interface contract for the Gitleaks Ignore File Editor (`gli-editor`). It defines invocation patterns, arguments, exit codes, and interactive mode behavior.

---

## Command Invocation

### Basic Syntax

```bash
gli-editor [OPTIONS] [FILE] [LINE_SPEC]
```

### Arguments

| Argument | Type | Required | Default | Description |
|----------|------|----------|---------|-------------|
| `FILE` | Path | No | `./.gitleaksignore` | Path to .gitleaksignore file |
| `LINE_SPEC` | Line specification | No | Full file | Line number or range to display |

### Options

| Option | Short | Long | Value | Description |
|--------|-------|------|-------|-------------|
| Help | `-h` | `--help` | None | Display help message and exit |
| Version | `-v` | `--version` | None | Display version and exit |
| Read-only | `-r` | `--read-only` | None | Launch in view-only mode (disable editing) |
| File path | `-f` | `--file <PATH>` | Path | Specify .gitleaksignore file path |
| Lines | `-l` | `--lines <SPEC>` | Line spec | Specify line range to display |
| Context | `-C` | `--context <NUM>` | Integer | Number of context lines around target (default: 3) |

---

## Line Specification Format

### Single Line

Display a specific line with context:

```bash
gli-editor 42              # Line 42 with 3 lines of context
gli-editor --lines 42      # Equivalent
```

### Line Range

Display a contiguous range of lines:

```bash
gli-editor 10-50           # Lines 10 through 50 (inclusive)
gli-editor --lines 10-50   # Equivalent
```

### Line with Custom Context

Display a line with specified context lines:

```bash
gli-editor 42+5            # Line 42 with 5 lines before and after
gli-editor --lines 42 -C 5 # Equivalent
```

### Special Values

| Value | Meaning |
|-------|---------|
| `1` or `:1` | Jump to first line |
| `$` or `:$` | Jump to last line |
| No spec | Display from beginning (full file) |

---

## Interactive Mode

When launched without `--read-only`, the application enters interactive TUI mode with keyboard controls.

### View Mode Keybindings

| Key | Action | Description |
|-----|--------|-------------|
| `j` / `↓` | Scroll down | Move viewport down by 1 line |
| `k` / `↑` | Scroll up | Move viewport up by 1 line |
| `d` / `PageDown` | Page down | Move viewport down by viewport height |
| `u` / `PageUp` | Page up | Move viewport up by viewport height |
| `g` / `Home` | Jump to top | Move to line 1 |
| `G` / `End` | Jump to bottom | Move to last line |
| `:<num>` | Jump to line | Move to specific line number |
| `i` / `Enter` | Edit current line | Enter edit mode for line under cursor |
| `?` / `F1` | Show help | Display keybinding reference |
| `q` / `Esc` | Quit | Exit application (prompt if unsaved changes) |

### Edit Mode Keybindings

| Key | Action | Description |
|-----|--------|-------------|
| `Esc` | Save and exit edit | Save changes and return to view mode |
| `Ctrl+C` | Cancel edit | Discard changes and return to view mode |
| `←` `→` | Move cursor | Navigate within line |
| `Home` | Cursor to start | Move cursor to beginning of line |
| `End` | Cursor to end | Move cursor to end of line |
| `Backspace` | Delete left | Remove character before cursor |
| `Delete` | Delete right | Remove character after cursor |
| `Ctrl+A` | Select all | Select entire line content |
| `Ctrl+K` | Delete to end | Delete from cursor to end of line |
| `Ctrl+U` | Delete to start | Delete from cursor to beginning of line |

### Conflict Resolution Modal

When concurrent modification is detected:

| Key | Action | Description |
|-----|--------|-------------|
| `R` | Reload file | Discard local changes, reload from disk |
| `O` | Overwrite | Force save local changes, ignore external modifications |
| `C` / `Esc` | Cancel | Return to edit mode without saving |

---

## Exit Codes

| Code | Meaning | Description |
|------|---------|-------------|
| `0` | Success | Operation completed successfully |
| `1` | File error | File not found, permission denied, or read error |
| `2` | Invalid arguments | Invalid command-line options or line specification |
| `3` | Write error | Unable to save changes (disk full, permissions) |
| `4` | Interrupted | User interrupted with Ctrl+C during critical operation |
| `64` | Usage error | Invalid invocation (use `--help` for usage) |

---

## Examples

### Example 1: Open default file

```bash
gli-editor
```

- Opens `./.gitleaksignore` in current directory
- Displays full file starting from line 1
- Interactive mode with view/edit capabilities

### Example 2: View specific line

```bash
gli-editor --lines 42
```

- Opens `./.gitleaksignore`
- Displays line 42 with 3 lines of context (39-45)
- Interactive mode

### Example 3: View line range

```bash
gli-editor -l 100-200
```

- Opens `./.gitleaksignore`
- Displays lines 100-200
- Interactive mode

### Example 4: Read-only mode

```bash
gli-editor --read-only --file /path/to/.gitleaksignore
```

- Opens specified file in read-only mode
- Editing disabled (no 'i' to enter edit mode)
- Useful for inspection without risk of accidental modifications

### Example 5: Custom context

```bash
gli-editor -l 50 -C 10
```

- Opens `./.gitleaksignore`
- Displays line 50 with 10 lines of context (40-60)
- Interactive mode

### Example 6: Help and version

```bash
gli-editor --help     # Display usage information
gli-editor --version  # Display version: gli-editor 1.0.0
```

---

## Output Format

### Help Message

```
gli-editor 1.0.0
Terminal editor for .gitleaksignore files

USAGE:
    gli-editor [OPTIONS] [FILE] [LINE_SPEC]

ARGS:
    <FILE>        Path to .gitleaksignore file [default: ./.gitleaksignore]
    <LINE_SPEC>   Line number or range (e.g., 42, 10-50, 42+5)

OPTIONS:
    -h, --help                 Print help information
    -v, --version              Print version information
    -r, --read-only            Launch in view-only mode (no editing)
    -f, --file <PATH>          Specify .gitleaksignore file path
    -l, --lines <SPEC>         Specify line range to display
    -C, --context <NUM>        Number of context lines [default: 3]

KEYBINDINGS (View Mode):
    j/↓          Scroll down          g/Home       Jump to top
    k/↑          Scroll up            G/End        Jump to bottom
    i/Enter      Edit line            ?/F1         Show help
    q/Esc        Quit application

KEYBINDINGS (Edit Mode):
    Esc          Save and exit        Ctrl+C       Cancel edit
    ←/→          Move cursor          Home/End     Line start/end

EXAMPLES:
    gli-editor                        # Open ./.gitleaksignore
    gli-editor --lines 42             # View line 42 with context
    gli-editor -l 100-200             # View lines 100-200
    gli-editor --read-only            # Open in read-only mode

For more information, visit: https://github.com/epy0n0ff/gli-editor
```

### Version Message

```
gli-editor 1.0.0
```

---

## Error Messages

### File Not Found

```
Error: File not found: /path/to/.gitleaksignore

Suggestion: Create the file with:
  touch .gitleaksignore

Exit code: 1
```

### Permission Denied

```
Error: Permission denied: /path/to/.gitleaksignore

Suggestion: Check file permissions with:
  ls -l /path/to/.gitleaksignore

Exit code: 1
```

### Invalid Line Specification

```
Error: Invalid line specification: '10-5'

Line range must be in format:
  - Single line: 42
  - Range: 10-50 (start must be ≤ end)
  - With context: 42+5

Exit code: 2
```

### Line Out of Bounds

```
Error: Line 1000 is out of bounds (file has 500 lines)

Valid range: 1-500

Exit code: 2
```

### Write Failure

```
Error: Unable to save changes: No space left on device

Backup preserved at: .gitleaksignore.backup.1735113600
Your changes are NOT lost. Free up disk space and retry.

Exit code: 3
```

### Concurrent Modification

```
Warning: File was modified by another process

Original: 2025-12-25 10:30:45
Current:  2025-12-25 10:35:12

Actions:
  [R] Reload file (discard your changes)
  [O] Overwrite (save your changes anyway)
  [C] Cancel (return to editing)

Choice: _
```

---

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `EDITOR` | (none) | Fallback editor if gli-editor crashes |
| `TERM` | (auto-detect) | Terminal type for rendering |
| `NO_COLOR` | (none) | If set, disable syntax highlighting |

---

## Configuration

### User Configuration File

Optional: `~/.config/gli-editor/config.toml`

```toml
[general]
default_context = 5           # Context lines (default: 3)
read_only = false             # Launch in read-only mode by default

[display]
show_line_numbers = true      # Show line numbers (always true for now)
syntax_highlighting = true    # Enable pattern-based coloring

[colors]
comment = "dark_gray"         # Comment line color
commit_hash = "yellow"        # Fingerprint commit hash color
file_path = "cyan"            # Fingerprint file path color
rule_id = "magenta"           # Fingerprint rule ID color
line_number = "green"         # Line number column color
invalid = "red"               # Invalid pattern color

[backup]
enabled = true                # Create backups before editing
max_backups = 5               # Keep last N backups
auto_cleanup = true           # Delete old backups automatically
```

**Note**: Configuration file is optional. All settings have sensible defaults.

---

## Interactive TUI Layout

### View Mode Layout

```
┌─────────────────────────────────────────────────────────────┐
│ gli-editor - .gitleaksignore (12,345 lines)        [VIEW]   │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│   100  cd5226711335c68be1e720b318b7bc3135a30eb2:cmd/file... │
│   101  # This is a comment                                   │
│   102                                                         │
│   103  9f067a02a5efa7110da117e80e8ea58d26847e70:docs/index...│
│   104  8a72ee99785fe3af5979d3e0a8cf6718841c244a:config/key...│
│   ...                                                         │
│                                                               │
├─────────────────────────────────────────────────────────────┤
│ VIEW | Line 103/12345 | j/k:scroll i:edit q:quit ?:help     │
└─────────────────────────────────────────────────────────────┘
```

### Edit Mode Layout

```
┌─────────────────────────────────────────────────────────────┐
│ gli-editor - .gitleaksignore (12,345 lines)        [EDIT]   │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│ Editing line 103:                                            │
│ ┌───────────────────────────────────────────────────────┐   │
│ │ cd5226711335c68be1e720b318b7bc3135a30eb2:cmd/file.go:█   │
│ └───────────────────────────────────────────────────────┘   │
│                                                               │
│ Original:                                                     │
│   cd5226711335c68be1e720b318b7bc3135a30eb2:cmd/file.go:se...│
│                                                               │
├─────────────────────────────────────────────────────────────┤
│ EDIT | Esc:save Ctrl+C:cancel ←/→:move cursor               │
└─────────────────────────────────────────────────────────────┘
```

---

## API Contract Summary

### Inputs

- **Command-line arguments**: File path, line specification, options
- **Interactive input**: Keyboard events (view navigation, editing)
- **File system**: .gitleaksignore file content

### Outputs

- **Terminal display**: Styled TUI with syntax highlighting
- **File system**: Modified .gitleaksignore file (on save)
- **Backup files**: Timestamped `.gitleaksignore.backup.*` files
- **Exit code**: Success/error status for scripting

### Side Effects

- Reads `.gitleaksignore` file
- Writes modified content to `.gitleaksignore` (on save)
- Creates backup files in same directory
- Updates file modification time (mtime) on save

### Guarantees

- **Atomicity**: File writes are atomic (temp file + rename)
- **Data safety**: Backup created before any modification
- **Encoding**: Preserves UTF-8 encoding
- **Line endings**: Preserves original line ending format (LF/CRLF)
- **Concurrent safety**: Detects external modifications before save

---

## Testing Contract

### Unit Test Cases

- Parse valid line specifications: `42`, `10-50`, `42+5`, `$`
- Reject invalid line specifications: `10-5`, `abc`, `-10`
- Validate file paths: relative, absolute, non-existent
- Handle special characters in file paths

### Integration Test Cases

- Launch with various argument combinations
- Navigate through file (scroll, jump)
- Edit line and save successfully
- Cancel edit without saving
- Detect concurrent modifications
- Create and cleanup backup files

### End-to-End Test Scenarios

1. **Happy Path**: Launch → View → Edit → Save → Exit
2. **Read-Only**: Launch with `--read-only`, verify editing disabled
3. **Concurrent Edit**: Open file, modify externally, attempt save, resolve conflict
4. **Large File**: Load 10,000-line file, verify <2s load time
5. **Error Handling**: Try to open non-existent file, verify error message

---

**Status**: Ready for implementation
