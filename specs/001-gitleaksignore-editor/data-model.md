# Data Model: Gitleaks Ignore File Editor

**Feature**: 001-gitleaksignore-editor
**Date**: 2025-12-25
**Status**: Design Phase

## Purpose

This document defines the core data structures and their relationships for the Gitleaks Ignore File Editor. All structures are technology-agnostic and focus on domain concepts.

---

## Core Entities

### 1. Ignore Pattern Entry

Represents a single line in the .gitleaksignore file.

**Attributes**:
- `line_number` (positive integer): 1-based line number in file
- `content` (string): Raw line content including any whitespace
- `pattern_type` (enum): Classification of the line content

**Pattern Type Values**:
- `Comment`: Line starting with `#` (after trimming whitespace)
- `Fingerprint`: Valid gitleaks fingerprint with 4 components
  - `commit_hash`: 40-character hexadecimal SHA-1 hash
  - `file_path`: Path to file (may contain `:` or `!` for archives)
  - `rule_id`: Kebab-case rule identifier (e.g., `generic-api-key`)
  - `line_number_in_file`: Integer line number where secret was found
- `BlankLine`: Empty or whitespace-only
- `Invalid`: Malformed entry that doesn't match other patterns

**Validation Rules**:
- Line number must be > 0
- Content must not contain null bytes
- Commit hash (if fingerprint) must be exactly 40 hex characters
- Line number in fingerprint (if present) must be parseable as u32

**State Transitions**: None (immutable after parsing)

---

### 2. Line Range

Represents a continuous sequence of lines for display or navigation.

**Attributes**:
- `start_line` (positive integer): First line number (1-based, inclusive)
- `end_line` (positive integer): Last line number (1-based, inclusive)
- `entries` (collection of IgnorePatternEntry): Lines within the range

**Validation Rules**:
- start_line ≤ end_line
- start_line ≥ 1
- end_line ≤ total file lines
- entries.length = (end_line - start_line + 1)

**Derived Values**:
- `total_lines`: end_line - start_line + 1
- `is_empty`: total_lines == 0

**Operations**:
- `expand_context(lines_before, lines_after)`: Create new range with additional context
- `contains_line(line_number)`: Check if line number is within range
- `get_entry(line_number)`: Retrieve entry at specific line

---

### 3. Edit Operation

Represents a modification to a specific line, used for tracking changes and undo functionality.

**Attributes**:
- `line_number` (positive integer): Target line (1-based)
- `original_content` (string): Content before edit
- `new_content` (string): Content after edit
- `timestamp` (datetime): When edit was performed
- `operation_type` (enum): Type of modification

**Operation Type Values**:
- `Update`: Modified existing line
- `Insert`: Added new line
- `Delete`: Removed line

**Validation Rules**:
- line_number must be valid for current file
- original_content must match current line content (for Update)
- new_content must preserve line ending format
- timestamp must be ≤ current time

**State Transitions**:
```
Pending → Applied → Saved
        ↓
      Reverted
```

**Operations**:
- `apply()`: Execute the edit operation on file content
- `revert()`: Restore original content
- `validate()`: Check if operation is still valid (no conflicts)

---

### 4. File Context

Represents the state of the .gitleaksignore file, including metadata for navigation and integrity checks.

**Attributes**:
- `file_path` (path): Absolute path to .gitleaksignore file
- `total_lines` (non-negative integer): Number of lines in file
- `line_ending_format` (enum): Detected line ending type
- `last_modified_time` (datetime): File system modification timestamp
- `current_view_position` (positive integer): Line number currently displayed
- `has_unsaved_changes` (boolean): Whether edits exist that aren't saved

**Line Ending Format Values**:
- `LF`: Unix-style `\n`
- `CRLF`: Windows-style `\r\n`
- `CR`: Legacy Mac `\r`

**Validation Rules**:
- file_path must exist and be readable
- total_lines ≥ 0 (empty file is valid)
- current_view_position ≤ total_lines (or 1 if empty)
- last_modified_time must be retrievable from filesystem

**State Transitions**:
```
Unloaded → Loaded → Modified → Saved
                    ↓           ↓
                  Reloaded ← Conflicted
```

**Operations**:
- `refresh_metadata()`: Update mtime and line count from filesystem
- `check_for_external_modifications()`: Compare current mtime with cached
- `mark_dirty()`: Set has_unsaved_changes = true
- `mark_clean()`: Set has_unsaved_changes = false

---

## Relationships

```
FileContext (1) ──contains──> (N) IgnorePatternEntry
                 ┃
                 └──tracks──> (N) EditOperation

LineRange (1) ──references──> (N) IgnorePatternEntry

EditOperation (1) ──modifies──> (1) IgnorePatternEntry
```

**Cardinality**:
- One FileContext contains zero or more IgnorePatternEntry instances (one per line)
- One FileContext tracks zero or more EditOperation instances (edit history)
- One LineRange references a subset of IgnorePatternEntry instances
- One EditOperation modifies exactly one IgnorePatternEntry

---

## Application State Model

### View Mode State

**Attributes**:
- `file_context`: FileContext instance
- `visible_range`: LineRange instance
- `scroll_offset`: Non-negative integer (viewport position)

**Invariants**:
- visible_range.start_line ≥ 1
- visible_range.end_line ≤ file_context.total_lines
- scroll_offset + viewport_height ≤ file_context.total_lines

**Operations**:
- `scroll_up()`: Decrease scroll_offset by 1 (min 0)
- `scroll_down()`: Increase scroll_offset by 1 (max limit)
- `jump_to_line(n)`: Set scroll_offset to center line n
- `enter_edit_mode(line_number)`: Transition to Edit Mode

---

### Edit Mode State

**Attributes**:
- `file_context`: FileContext instance
- `target_line_number`: Positive integer (line being edited)
- `original_content`: String (backup for cancel operation)
- `current_content`: String (live editing buffer)
- `cursor_position`: Non-negative integer (character offset)

**Invariants**:
- target_line_number ≤ file_context.total_lines
- cursor_position ≤ current_content.length
- original_content matches file_context.entries[target_line_number] at entry time

**Operations**:
- `update_content(new_text)`: Modify current_content
- `save()`: Create EditOperation, apply to file_context, transition to View Mode
- `cancel()`: Discard changes, transition to View Mode
- `has_changes()`: Compare current_content with original_content

---

## Data Validation

### On File Load

1. **Encoding Check**: Verify UTF-8 encoding (reject binary files)
2. **Line Ending Detection**: Scan for `\r\n`, `\n`, or `\r`
3. **Pattern Parsing**: Classify each line as Comment/Fingerprint/Blank/Invalid
4. **Metadata Capture**: Store mtime, file size, line count

### On Edit Operation

1. **Line Range Check**: Ensure target line exists
2. **Concurrent Modification Check**: Compare current mtime with cached
3. **Content Validation**: No null bytes, valid UTF-8
4. **Line Ending Consistency**: Apply detected line ending format

### On File Save

1. **Backup Creation**: Copy current file with timestamp
2. **Atomic Write**: Write to temp file, then rename
3. **Line Ending Preservation**: Use detected format
4. **Metadata Update**: Refresh mtime after successful write

---

## Error States

### File-Level Errors

- `FileNotFound`: .gitleaksignore doesn't exist at specified path
- `PermissionDenied`: Insufficient permissions to read/write file
- `InvalidEncoding`: File contains non-UTF-8 bytes
- `FileTooLarge`: File exceeds maximum line count (implementation limit)

### Edit-Level Errors

- `LineOutOfBounds`: Target line number > total lines
- `ConcurrentModification`: File mtime changed during edit session
- `InvalidContent`: New content contains null bytes or invalid UTF-8
- `WriteFailure`: Unable to save changes (disk full, permissions)

### Recovery Actions

| Error | Recovery Strategy |
|-------|------------------|
| FileNotFound | Offer to create new file with initial content |
| PermissionDenied | Display error, suggest checking file permissions |
| InvalidEncoding | Refuse to load, recommend text editor for fixing |
| ConcurrentModification | Prompt user: Reload / Overwrite / Cancel |
| WriteFailure | Keep backup, display error with backup location |

---

## Performance Considerations

### Memory Footprint (10,000 lines)

| Component | Per-Line Size | Total for 10k Lines |
|-----------|--------------|---------------------|
| Raw line content | ~50 bytes | ~500 KB |
| Pattern type enum | ~8 bytes | ~80 KB |
| Line metadata | ~16 bytes | ~160 KB |
| Visible viewport (50 lines) | ~55 bytes styled | ~2.75 KB |
| **Total in-memory** | | **~740 KB + 2.75 KB viewport** |

### Access Patterns

- **Sequential Read**: O(n) for full file parse (on load only)
- **Random Access**: O(1) for line lookup by number (array indexing)
- **Range Query**: O(k) where k = range size (viewport rendering)
- **Edit Operation**: O(1) for single line modification

### Caching Strategy

**Cache Pattern Types on Load**:
- Parse each line once during file load
- Store PatternType alongside content
- Avoid re-parsing on every render (50x speedup for viewport)

**Don't Cache**:
- Styled spans (cheap to regenerate, 200x memory overhead)
- Viewport content (changes frequently with scrolling)

---

## State Machine: Application Modes

```
┌─────────────────────────────────────────────────────────┐
│                     Uninitialized                        │
└───────────────────────┬─────────────────────────────────┘
                        │ load_file()
                        ▼
┌─────────────────────────────────────────────────────────┐
│                     View Mode                            │
│  - Navigate file (scroll, jump)                          │
│  - Syntax-highlighted display                            │
│  - Read-only access                                      │
└───┬───────────────────────────────────────────────┬─────┘
    │ enter_edit_mode(line)                         │ quit()
    ▼                                                ▼
┌──────────────────────────────────┐          ┌─────────┐
│        Edit Mode                 │          │  Exit   │
│  - Single-line text editing      │          └─────────┘
│  - Cursor management             │
│  - Save/cancel options           │
└─┬──────────────────────────────┬─┘
  │ save()                        │ cancel()
  │                               │
  │ ┌─────────────────────────┐  │
  │ │  Concurrent Mod Check   │  │
  │ │  - mtime comparison     │  │
  │ └───┬──────────────┬──────┘  │
  │     │ OK           │ Conflict│
  │     ▼              ▼          │
  │ ┌────────┐    ┌─────────────┐│
  │ │ Save   │    │ Conflict    ││
  │ │Success │    │  Modal      ││
  │ └───┬────┘    └──┬──────────┘│
  │     │            │ resolve    │
  │     └────────────┴────────────┘
  ▼
Back to View Mode
```

---

## Example Data Flows

### Flow 1: View Specific Line Range

```
1. User Input: "Show lines 100-150"
   ├─> Validate: 100 ≤ 150, both ≤ total_lines
   └─> Create: LineRange { start: 100, end: 150 }

2. Retrieve Entries:
   ├─> Access: file_context.entries[99..149] (0-indexed)
   └─> Collect: 50 IgnorePatternEntry instances

3. Render Viewport:
   ├─> For each entry: Generate styled spans based on pattern_type
   ├─> Compose: Line number + content spans
   └─> Display: Ratatui Paragraph widget

4. Update State:
   └─> Set: file_context.current_view_position = 100
```

### Flow 2: Edit Line

```
1. User Action: Press 'i' on line 200
   ├─> Validate: line 200 exists
   ├─> Capture: original_content = entries[199].content
   └─> Transition: View Mode → Edit Mode

2. User Edits: Modify text content
   ├─> Update: current_content buffer
   └─> Render: TextArea widget with cursor

3. User Saves: Press Esc
   ├─> Check: file_context.check_for_external_modifications()
   │   └─> If modified: Show conflict modal (see Flow 3)
   ├─> Create: EditOperation { line: 200, original, new, timestamp }
   ├─> Apply: entries[199].content = new_content
   ├─> Save: Atomic write with backup
   └─> Transition: Edit Mode → View Mode

4. Update Metadata:
   ├─> Refresh: file_context.last_modified_time
   └─> Mark: file_context.has_unsaved_changes = false
```

### Flow 3: Concurrent Modification Resolution

```
1. Save Attempt: file_context.check_for_external_modifications() returns true
   └─> Display: Conflict modal with options

2. User Choice: [R]eload / [O]verwrite / [C]ancel
   ├─> Reload:
   │   ├─> Discard: current edit operation
   │   ├─> Reload: file_context.load()
   │   └─> Return: View Mode at same position
   │
   ├─> Overwrite:
   │   ├─> Proceed: Save despite conflict (user accepts risk)
   │   ├─> Backup: Old file preserved with timestamp
   │   └─> Save: Atomic write
   │
   └─> Cancel:
       └─> Return: Edit Mode (user continues editing)
```

---

## Testing Considerations

### Data Model Unit Tests

- **Entity Construction**: Valid and invalid attribute combinations
- **Validation Rules**: Boundary conditions (line 0, negative values)
- **State Transitions**: Valid and invalid mode changes
- **Pattern Parsing**: All pattern types including edge cases

### Integration Tests

- **File Load**: Empty file, single line, 10,000 lines, mixed line endings
- **Edit Workflow**: View → Edit → Save → View round-trip
- **Concurrent Modification**: Simulate external file change during edit
- **Backup Recovery**: Verify backup creation and restoration

### Performance Tests

- **Load Time**: 10,000-line file in <2s (SC-001)
- **Memory Usage**: <10MB for 10,000 lines
- **Viewport Render**: <1ms for 50-line viewport
- **Edit Latency**: <200ms from keypress to screen update

---

**Status**: Ready for implementation based on research findings in research.md
