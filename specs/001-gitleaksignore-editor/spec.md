# Feature Specification: Gitleaks Ignore File Editor

**Feature Branch**: `001-gitleaksignore-editor`
**Created**: 2025-12-25
**Status**: Draft
**Input**: User description: "gitleaksignoreファイルの指定した行数のコードをプレビューして編集を補助するterminalアプリを作成してください。"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - View Specific Line Ranges (Priority: P1)

A developer needs to quickly view and understand specific lines in their .gitleaksignore file to determine if certain patterns are correctly configured. They want to inspect line ranges without opening a full text editor.

**Why this priority**: This is the core value proposition - providing quick, focused access to specific parts of the file. Without this, the application has no purpose.

**Independent Test**: Can be fully tested by launching the application with a line number or range, viewing the displayed content, and confirming it matches the actual file contents. Delivers immediate value for inspection tasks.

**Acceptance Scenarios**:

1. **Given** a .gitleaksignore file exists in the current directory, **When** user specifies a single line number (e.g., line 15), **Then** system displays that line with surrounding context
2. **Given** a .gitleaksignore file exists, **When** user specifies a line range (e.g., lines 10-25), **Then** system displays all lines in that range with line numbers
3. **Given** user is viewing a line range, **When** the file content is displayed, **Then** line numbers are shown alongside each line for easy reference
4. **Given** user requests a line number beyond the file length, **When** the application processes the request, **Then** system displays an appropriate message indicating the line doesn't exist

---

### User Story 2 - Quick Pattern Editing (Priority: P2)

A developer identifies an incorrect or outdated pattern in their .gitleaksignore file and needs to modify it quickly without leaving the terminal or opening a full-featured editor.

**Why this priority**: Editing capabilities transform this from a read-only viewer to a productivity tool. However, viewing must work first before editing is useful.

**Independent Test**: Can be tested by viewing a line, initiating an edit operation, modifying the content, and verifying the change is saved to the file. Delivers value for quick corrections.

**Acceptance Scenarios**:

1. **Given** user is viewing a specific line, **When** user initiates edit mode for that line, **Then** system allows inline editing of the line content
2. **Given** user has modified a line, **When** user confirms the change, **Then** system updates the .gitleaksignore file with the new content
3. **Given** user is editing a line, **When** user cancels the edit operation, **Then** system discards changes and returns to viewing mode without modifying the file
4. **Given** user completes an edit, **When** the file is updated, **Then** system displays a confirmation message with the updated line

---

### User Story 3 - Navigate and Browse File (Priority: P3)

A developer wants to explore their .gitleaksignore file section by section to understand the overall structure and review multiple patterns without specifying exact line numbers upfront.

**Why this priority**: Navigation enhances usability but isn't essential for the core use case of viewing specific lines. Users can always re-run the command with different line numbers.

**Independent Test**: Can be tested by launching the viewer, using navigation commands to move through the file, and confirming smooth browsing experience. Delivers value for exploratory workflows.

**Acceptance Scenarios**:

1. **Given** user is viewing a line range, **When** user triggers "next section" navigation, **Then** system displays the next range of lines
2. **Given** user is viewing lines beyond the first range, **When** user triggers "previous section" navigation, **Then** system displays the previous range of lines
3. **Given** user is at the end of the file, **When** user attempts to navigate forward, **Then** system indicates no more content is available
4. **Given** user is browsing the file, **When** user requests to jump to a specific line number, **Then** system immediately displays that line with context

---

### User Story 4 - Syntax Highlighting and Pattern Recognition (Priority: P4)

A developer viewing .gitleaksignore patterns wants visual cues to distinguish different types of content (comments, patterns, paths) to quickly understand the file structure.

**Why this priority**: This is a quality-of-life enhancement that improves readability but isn't required for basic functionality. Users can still accomplish their tasks without it.

**Independent Test**: Can be tested by viewing a file with various pattern types and verifying that different elements are visually distinguished. Delivers value for improved comprehension.

**Acceptance Scenarios**:

1. **Given** user is viewing file content, **When** the display renders, **Then** comments (lines starting with #) are visually distinguished from patterns
2. **Given** file contains various pattern types, **When** content is displayed, **Then** file paths, regular expressions, and commit hashes are distinguishable
3. **Given** user views a malformed pattern, **When** content is displayed, **Then** system highlights potential syntax issues

---

### User Story 5 - Visual Cursor Indicator (Priority: P4)

A developer navigating through the .gitleaksignore file needs to clearly identify which line is currently selected for editing or deletion operations.

**Why this priority**: Clear visual feedback enhances usability but isn't essential for core functionality. Users can still navigate and edit without it, though it may be less intuitive.

**Independent Test**: Can be tested by navigating through file lines and confirming that the current line is visually distinct from others.

**Acceptance Scenarios**:

1. **Given** user is viewing file content, **When** cursor is on a specific line, **Then** system displays a visual indicator (cursor symbol) next to the line number
2. **Given** user navigates between lines, **When** cursor position changes, **Then** system updates the visual indicator to follow the cursor
3. **Given** user is viewing the current line, **When** the line is highlighted, **Then** system applies background color and bold styling to distinguish it from other lines

---

### User Story 6 - Live Preview Pane for Fingerprints (Priority: P3)

A developer reviewing fingerprint entries in .gitleaksignore wants to see the actual source code that triggered the pattern to understand context and decide whether the ignore rule is still valid.

**Why this priority**: Preview functionality significantly enhances the review process but requires fingerprint parsing to work first. Users can still edit patterns without it.

**Independent Test**: Can be tested by selecting a fingerprint entry and verifying that the referenced source file and line are displayed in a split pane.

**Acceptance Scenarios**:

1. **Given** user selects a fingerprint entry, **When** the entry contains file path and line number, **Then** system displays the source file content in a preview pane
2. **Given** preview pane is visible, **When** source file is displayed, **Then** system highlights the target line referenced in the fingerprint
3. **Given** user navigates between fingerprint entries, **When** cursor moves to a new entry, **Then** system automatically updates the preview content
4. **Given** user is viewing a preview, **When** user toggles preview mode, **Then** system shows or hides the preview pane while preserving full-width viewing mode
5. **Given** fingerprint references a non-existent file or line number, **When** preview is requested, **Then** system handles the error gracefully without crashing

---

### User Story 7 - Line Deletion (Priority: P2)

A developer reviewing .gitleaksignore entries identifies obsolete patterns that should be removed and wants to delete them directly without opening a full text editor.

**Why this priority**: Deletion complements editing capabilities and is essential for file maintenance. However, it's slightly lower priority than editing since users can delete by editing the line to be empty.

**Independent Test**: Can be tested by selecting a line, executing delete command, and verifying the line is removed from the file with proper backup creation.

**Acceptance Scenarios**:

1. **Given** user is viewing a specific line, **When** user executes delete command (dd or Delete key), **Then** system removes the line from the file
2. **Given** user deletes a line, **When** deletion is confirmed, **Then** system creates a backup before removing the line
3. **Given** user accidentally triggers delete, **When** using dd command, **Then** system requires confirmation (second 'd' press) to prevent accidental deletion
4. **Given** user deletes the last line in file, **When** deletion completes, **Then** system adjusts cursor to the new last line or handles empty file state
5. **Given** file is in read-only mode, **When** user attempts to delete, **Then** system displays an error message and prevents deletion
6. **Given** user deletes a line, **When** deletion completes, **Then** system renumbers all subsequent lines and updates the preview

---

### Edge Cases

- What happens when the .gitleaksignore file doesn't exist in the current directory?
- How does the system handle an empty .gitleaksignore file?
- What happens when user specifies a negative line number or zero?
- How does the system handle a line range where start > end (e.g., lines 50-20)?
- What happens when the file is modified by another process while the application is running?
- How does the system handle very large .gitleaksignore files (e.g., 10,000+ lines)?
- What happens when the file contains non-UTF-8 characters or binary content?
- How does the system handle files without proper line endings or mixed line ending types?
- What happens when the user lacks read permissions for the .gitleaksignore file?
- How does the system handle concurrent edits by multiple users?
- What happens when a fingerprint entry references a file that doesn't exist?
- How does the system handle fingerprints with line numbers exceeding the target file's length?
- What happens when user deletes the last line in the file?
- How does the system handle page navigation when reaching the end of file?
- What happens when terminal size changes during execution?
- How does the system handle fingerprints with file paths containing colons (e.g., archives)?
- What happens when cursor is at the bottom of viewport during smooth scrolling?
- How does the system handle empty file state after all lines are deleted?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST locate and read .gitleaksignore files from the specified directory or current working directory
- **FR-002**: System MUST display individual lines or line ranges with accurate line numbers
- **FR-003**: System MUST allow users to specify line numbers or ranges via command-line arguments or interactive input
- **FR-004**: System MUST provide context lines around requested line numbers (e.g., ±3 lines) for better understanding
- **FR-005**: System MUST support interactive editing of displayed lines
- **FR-006**: System MUST preserve existing file formatting and line endings when saving edits
- **FR-007**: System MUST validate that line numbers are within valid bounds before processing
- **FR-008**: System MUST display appropriate error messages when .gitleaksignore file is not found
- **FR-009**: System MUST create a backup of the original file before applying edits
- **FR-010**: System MUST support navigation between different line ranges within the same session
- **FR-011**: System MUST display help information explaining available commands and usage
- **FR-012**: System MUST allow users to exit the application gracefully at any time
- **FR-013**: System MUST handle keyboard interrupts (Ctrl+C) safely without corrupting the file
- **FR-014**: System MUST distinguish between different content types (comments, patterns, blank lines)
- **FR-015**: System MUST support both viewing-only mode and editing mode
- **FR-016**: System MUST display a visual cursor indicator showing the currently selected line
- **FR-017**: System MUST parse fingerprint entries to extract file path, line number, rule ID, and optional commit hash
- **FR-018**: System MUST display a live preview pane showing source file content for fingerprint entries
- **FR-019**: System MUST support toggling preview pane visibility on/off
- **FR-020**: System MUST handle fingerprint entries in both full format (with commit hash) and short format (without commit hash)
- **FR-021**: System MUST support line deletion with automatic backup creation
- **FR-022**: System MUST require confirmation for deletion operations (dd command requires two 'd' presses)
- **FR-023**: System MUST renumber all lines after deletion operations
- **FR-024**: System MUST handle empty file state after deleting the last line
- **FR-025**: System MUST adjust viewport to terminal size for proper scrolling
- **FR-026**: System MUST implement smooth scrolling with scroll margin to prevent cursor from reaching screen edges

### Key Entities

- **Ignore Pattern Entry**: Represents a single line in the .gitleaksignore file, containing either a pattern (file path, regex, commit hash), a comment, or blank space. Attributes include line number, content, and type (pattern/comment/blank).
- **Line Range**: Represents a continuous sequence of lines to be displayed, with start line number, end line number, and the associated content. Used for viewing and navigation operations.
- **Edit Operation**: Represents a modification to a specific line, including original content, new content, line number, and timestamp. Used for tracking changes and enabling undo functionality.
- **File Context**: Represents the state of the .gitleaksignore file, including file path, total line count, last modified timestamp, and current view position. Used for navigation and file integrity checks.
- **Preview Content**: Represents the source file content referenced by a fingerprint entry, including target file path, target line number, displayed lines (with context), and start line number for the preview window.
- **View State**: Represents the current view configuration, including visible line range, current cursor position, scroll offset, preview enabled status, and preview content.
- **Delete Operation**: Represents a line deletion operation, including deleted line number, original content, and backup file path. Used for tracking deletions and maintaining file integrity.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can view any line or line range in under 2 seconds from application launch
- **SC-002**: Users can complete a single-line edit (view, modify, save) in under 30 seconds
- **SC-003**: 95% of view and edit operations complete successfully without errors for valid inputs
- **SC-004**: System correctly handles files up to 10,000 lines without performance degradation (response time remains under 2 seconds)
- **SC-005**: Zero data loss incidents - all successful edit operations correctly update the file without corruption
- **SC-006**: Users can identify pattern types (comments vs actual patterns) within 3 seconds of viewing due to visual distinction
- **SC-007**: Error messages are clear enough that 90% of users can resolve common issues (file not found, invalid line numbers) without external help
- **SC-008**: Application startup time is under 1 second on standard hardware

## Assumptions *(optional)*

- Users are familiar with basic terminal/command-line operations
- The .gitleaksignore file follows standard text file conventions (UTF-8 encoding, standard line endings)
- Users have appropriate file system permissions to read and write .gitleaksignore files
- The application will be used primarily for quick inspections and minor edits, not as a replacement for full-featured text editors
- Gitleaks ignore patterns follow documented gitleaks syntax conventions
- Users typically work with .gitleaksignore files under 1,000 lines
- The terminal environment supports basic text rendering and keyboard input
- File modifications are generally performed by one user at a time (no complex concurrent editing scenarios)

## Dependencies *(optional)*

- Access to file system for reading and writing .gitleaksignore files
- Terminal environment with standard input/output capabilities
- File system that supports atomic writes or safe file replacement operations for data integrity
- Understanding of .gitleaksignore file format and gitleaks pattern syntax

## Out of Scope *(optional)*

- Integration with gitleaks validation engine (pattern testing against actual secrets)
- Git integration features (committing changes, viewing git history)
- Multi-file editing or project-wide pattern management
- Remote file editing (files not on local file system)
- Real-time collaboration features
- Pattern suggestion or auto-completion based on repository content
- Diff visualization for comparing changes before/after edits
- Full text editor features (search/replace across entire file, regex find, etc.)
- Support for other ignore file formats (.gitignore, .dockerignore, etc.)
- Graphical user interface version
