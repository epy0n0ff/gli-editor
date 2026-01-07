# Implementation Plan: Gitleaks Ignore File Editor

**Branch**: `001-gitleaksignore-editor` | **Date**: 2025-12-25 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-gitleaksignore-editor/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

A terminal-based application for viewing and editing specific line ranges in .gitleaksignore files. The application provides quick access to line-specific content with interactive editing capabilities, file navigation, and syntax highlighting for different pattern types (comments, paths, regex patterns). Built in Rust using Ratatui for terminal UI rendering.

## Technical Context

**Language/Version**: Rust 1.75+ (latest stable)
**Primary Dependencies**: ratatui (https://github.com/ratatui/ratatui) for TUI, crossterm for terminal backend
**Storage**: File system operations (read/write .gitleaksignore files)
**Testing**: cargo test with unit tests, integration tests for file operations
**Target Platform**: Cross-platform terminal (Linux, macOS, Windows)
**Project Type**: Single binary CLI application with interactive TUI mode
**Performance Goals**: <1s startup time, <2s for viewing any line range up to 10,000 lines
**Constraints**: <200ms UI response time for user interactions, safe file operations with backups, UTF-8 encoding support
**Scale/Scope**: Handle files up to 10,000 lines efficiently, single-user interactive sessions

**Research Required**:
- NEEDS CLARIFICATION: Ratatui widget selection for editable text fields (input widgets vs text editor widgets)
- NEEDS CLARIFICATION: File backup strategy (temporary files, versioning approach)
- NEEDS CLARIFICATION: Line ending preservation strategy (LF, CRLF, mixed)
- NEEDS CLARIFICATION: Syntax highlighting implementation (pattern parsing vs regex-based)
- NEEDS CLARIFICATION: Error recovery strategy for concurrent file modifications

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

**Status**: ⚠️ PENDING - Constitution file contains only template placeholders

**Note**: The constitution file (`.specify/memory/constitution.md`) currently contains only template structure without concrete principles. This plan will proceed with industry-standard best practices:

- **Library-First Design**: Core logic separated from TUI layer for testability
- **Test Coverage**: Unit tests for file operations, integration tests for user flows
- **Error Handling**: Comprehensive error types with user-friendly messages
- **Documentation**: Inline docs, README with usage examples

**Re-evaluation Required**: After constitution is defined, verify alignment with project principles.

## Project Structure

### Documentation (this feature)

```text
specs/001-gitleaksignore-editor/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
│   └── cli-interface.md # Command-line interface specification
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src/
├── main.rs              # Entry point, CLI argument parsing
├── app.rs               # Application state and main loop
├── ui/                  # Ratatui UI components
│   ├── mod.rs
│   ├── viewer.rs        # Line viewing widget
│   ├── editor.rs        # Inline editing widget
│   ├── navigation.rs    # Navigation controls
│   └── styles.rs        # Syntax highlighting styles
├── core/                # Business logic (library layer)
│   ├── mod.rs
│   ├── file_reader.rs   # File I/O operations
│   ├── line_parser.rs   # Pattern type detection
│   ├── editor.rs        # Edit operation management
│   └── backup.rs        # Backup file handling
├── models/              # Data structures
│   ├── mod.rs
│   ├── line.rs          # Line and LineRange types
│   ├── pattern.rs       # Pattern type enum
│   └── edit.rs          # Edit operation type
└── error.rs             # Error types and handling

tests/
├── integration/         # End-to-end file operation tests
│   ├── mod.rs
│   ├── view_tests.rs    # Test viewing scenarios
│   ├── edit_tests.rs    # Test editing scenarios
│   └── navigation_tests.rs
└── unit/                # Unit tests for core logic
    ├── file_reader_tests.rs
    ├── line_parser_tests.rs
    └── backup_tests.rs

Cargo.toml               # Dependencies and project metadata
README.md                # User documentation
```

**Structure Decision**: Single binary application structure. The `src/core/` directory contains testable business logic independent of UI framework, while `src/ui/` contains Ratatui-specific rendering code. This separation enables:
- Unit testing of file operations without UI
- Integration testing of complete user scenarios
- Potential future support for alternative interfaces (pure CLI mode)

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

N/A - No constitutional violations identified. Standard single-binary Rust application structure.

## Phase 0: Research & Decisions

**Objective**: Resolve all NEEDS CLARIFICATION items in Technical Context

### Research Tasks

1. **Ratatui Widget Selection**
   - Investigate `tui-textarea` crate for multi-line editing
   - Evaluate built-in `Paragraph` vs `List` widgets for viewing
   - Determine keyboard event handling strategy
   - **Output**: Widget recommendations in research.md

2. **File Backup Strategy**
   - Research atomic file write patterns in Rust
   - Evaluate `.gitleaksignore.bak` vs timestamped backups
   - Determine cleanup strategy (auto-delete on success?)
   - **Output**: Backup approach specification in research.md

3. **Line Ending Preservation**
   - Research `std::io::BufRead` line ending handling
   - Evaluate auto-detection vs explicit configuration
   - Determine preservation strategy during edits
   - **Output**: Line ending handling approach in research.md

4. **Syntax Highlighting Implementation**
   - Define .gitleaksignore pattern syntax rules
   - Evaluate regex-based vs hand-written parser
   - Research Ratatui styling capabilities (`Style`, `Color`)
   - **Output**: Syntax highlighting design in research.md

5. **Concurrent Modification Detection**
   - Research file modification time checking (mtime)
   - Evaluate file locking vs detection-only approach
   - Determine user notification strategy
   - **Output**: Error recovery specification in research.md

### Research Output

All findings will be consolidated in `specs/001-gitleaksignore-editor/research.md` with:
- **Decision**: What was chosen
- **Rationale**: Why chosen (performance, safety, user experience)
- **Alternatives Considered**: What else was evaluated
- **Implementation Notes**: Key technical details

## Phase 1: Design & Contracts

**Prerequisites**: research.md complete

### Deliverables

1. **data-model.md**: Core data structures
   - `IgnorePatternEntry` (line number, content, pattern type)
   - `LineRange` (start, end, content buffer)
   - `EditOperation` (line number, old content, new content, timestamp)
   - `FileContext` (path, total lines, current position, last modified)
   - State transitions for view mode ↔ edit mode

2. **contracts/cli-interface.md**: Command-line interface
   - Invocation patterns: `gli-editor [OPTIONS] [FILE] [LINE_SPEC]`
   - Arguments: `--file`, `--lines`, `--read-only`, `--help`
   - Line specification format: `N` (single line), `N-M` (range), `N+C` (N with context)
   - Exit codes: 0 (success), 1 (file error), 2 (invalid args)
   - Interactive mode keyboard bindings

3. **quickstart.md**: Developer onboarding
   - Clone and build instructions
   - Running tests (`cargo test`)
   - Example usage scenarios
   - Development workflow (TDD approach)
   - Contributing guidelines

### Agent Context Update

Run `.specify/scripts/bash/update-agent-context.sh claude` to update AI agent context with:
- Rust + Ratatui technology stack
- Project structure and conventions
- File operation patterns
- TUI best practices

## Phase 2: Task Generation

**Prerequisites**: Phase 1 complete

**Not executed by `/speckit.plan`** - Use `/speckit.tasks` command to generate `tasks.md` with:
- Dependency-ordered implementation tasks
- Acceptance criteria per task
- Estimated complexity
- Test requirements

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Ratatui learning curve | Medium | Medium | Allocate time for examples, use tui-rs tutorials |
| File corruption during edits | Low | High | Atomic writes with backups, comprehensive tests |
| Terminal compatibility issues | Medium | Medium | Test on Linux/macOS/Windows, use crossterm abstraction |
| Performance with large files | Low | Medium | Lazy loading, viewport-based rendering |
| UTF-8 handling edge cases | Medium | Low | Use Rust's built-in UTF-8 validation, handle errors gracefully |

## Success Metrics Mapping

Mapping specification success criteria to implementation checkpoints:

- **SC-001** (view in <2s): Benchmark file reading and rendering performance
- **SC-002** (edit in <30s): Measure average time for edit workflow in integration tests
- **SC-003** (95% success rate): Track test pass rate, error handling coverage
- **SC-004** (10k lines no degradation): Load testing with large files
- **SC-005** (zero data loss): Verify backup system in integration tests
- **SC-006** (identify patterns in 3s): User testing of syntax highlighting
- **SC-007** (90% self-resolve errors): Review error message clarity
- **SC-008** (startup <1s): Binary size and initialization time benchmarks

## Next Steps

1. ✅ Complete Phase 0: Run research tasks and generate `research.md`
2. ⏳ Complete Phase 1: Generate `data-model.md`, `contracts/`, `quickstart.md`
3. ⏳ Update agent context with technology stack
4. ⏳ Use `/speckit.tasks` to generate implementation tasks
5. ⏳ Use `/speckit.implement` to execute tasks
