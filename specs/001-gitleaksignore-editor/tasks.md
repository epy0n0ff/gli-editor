# Tasks: Gitleaks Ignore File Editor

**Input**: Design documents from `/specs/001-gitleaksignore-editor/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/cli-interface.md

**Tests**: Not explicitly requested in specification - focusing on implementation tasks only

**Organization**: Tasks are grouped by user story (P1-P4) to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3, US4)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic Rust structure per plan.md

- [X] T001 Initialize Rust project with cargo init --name gli-editor
- [X] T002 [P] Configure Cargo.toml with dependencies: ratatui 0.26, crossterm 0.27, tui-textarea 0.4, tempfile 3.8, anyhow 1.0, clap 4.5
- [X] T003 [P] Create project directory structure: src/models/, src/core/, src/ui/, tests/integration/, tests/unit/ per plan.md
- [X] T004 [P] Setup rustfmt.toml and clippy configuration for code style
- [X] T005 [P] Create README.md with project description and usage from quickstart.md
- [X] T006 [P] Setup .gitignore for Rust projects (target/, Cargo.lock for libraries)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story implementation

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T007 Create error types module in src/error.rs with FileError, ParseError, EditError enums per data-model.md
- [X] T008 [P] Implement PatternType enum in src/models/pattern.rs with Comment, Fingerprint, BlankLine, Invalid variants per data-model.md
- [X] T009 [P] Implement LineEnding enum in src/core/file_reader.rs with LF, CRLF, CR variants and detect() method per research.md
- [X] T010 Create IgnorePatternEntry struct in src/models/line.rs with line_number, content, pattern_type fields per data-model.md
- [X] T011 Implement PatternType::parse() method in src/models/pattern.rs using hand-written parser per research.md
- [X] T012 Create FileContext struct in src/core/file_reader.rs with file_path, total_lines, line_ending_format, last_modified_time fields per data-model.md
- [X] T013 Implement file reading with line ending preservation in src/core/file_reader.rs using BufRead::read_line() per research.md
- [X] T014 [P] Create BackupManager struct in src/core/backup.rs for timestamped backup creation per research.md
- [X] T015 [P] Implement atomic file write using tempfile::NamedTempFile in src/core/file_reader.rs per research.md
- [X] T016 Implement FileSnapshot struct in src/core/file_reader.rs for concurrent modification detection using mtime per research.md

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - View Specific Line Ranges (Priority: P1) 🎯 MVP

**Goal**: Enable viewing of specific lines or line ranges in .gitleaksignore files with syntax highlighting

**Independent Test**: Launch application with line specification, verify displayed content matches file, confirm line numbers shown

**Acceptance Criteria**:
- Display single line with context (e.g., line 15 with ±3 lines)
- Display line range (e.g., lines 10-25)
- Show line numbers alongside content
- Handle out-of-bounds line numbers with clear error message

### Implementation for User Story 1

- [X] T017 [P] [US1] Create LineRange struct in src/models/line.rs with start_line, end_line, entries fields per data-model.md
- [X] T018 [P] [US1] Implement CLI argument parsing in src/main.rs using clap or similar for --file, --lines, --context options per contracts/cli-interface.md
- [X] T019 [US1] Implement line specification parser in src/main.rs for formats: single (42), range (10-50), with context (42+5) per contracts/cli-interface.md
- [X] T020 [US1] Create ViewState struct in src/app.rs with file_context, visible_range, scroll_offset per data-model.md
- [X] T021 [US1] Implement syntax highlighting logic in src/ui/viewer.rs with color scheme: comments (dark gray), commit hash (yellow), file path (cyan), rule ID (magenta) per research.md
- [X] T022 [US1] Create viewport rendering function in src/ui/viewer.rs using Paragraph widget for line display per research.md
- [X] T023 [US1] Implement line number formatting in src/ui/viewer.rs with right-aligned 4-digit format per research.md
- [X] T024 [US1] Add file validation in src/core/file_reader.rs: check file exists, readable, UTF-8 encoding per data-model.md
- [X] T025 [US1] Implement line bounds validation in src/core/file_reader.rs: validate line numbers within 1 to total_lines per data-model.md
- [X] T026 [US1] Create error message formatting in src/error.rs for FileNotFound, PermissionDenied, InvalidLineSpec, LineOutOfBounds per contracts/cli-interface.md
- [X] T027 [US1] Wire up main application loop in src/main.rs: parse args, load file, create view state, render initial view

**Checkpoint**: At this point, users can view .gitleaksignore files with syntax highlighting - US1 complete and testable

---

## Phase 4: User Story 2 - Quick Pattern Editing (Priority: P2)

**Goal**: Enable interactive editing of specific lines with safe file updates

**Independent Test**: View a line, enter edit mode, modify content, save, verify file updated correctly with backup created

**Acceptance Criteria**:
- Enter edit mode for current line
- Inline text editing with cursor
- Save changes to file atomically
- Cancel edit discards changes
- Display confirmation after save

### Implementation for User Story 2

- [X] T028 [P] [US2] Create EditOperation struct in src/models/edit.rs with line_number, original_content, new_content, timestamp, operation_type per data-model.md
- [X] T029 [P] [US2] Create EditState struct in src/app.rs with textarea, original_line, original_content per data-model.md
- [X] T030 [P] [US2] Implement AppMode enum in src/app.rs with View and Edit variants per data-model.md
- [X] T031 [US2] Integrate tui-textarea widget in src/ui/viewer.rs for single-line editing per research.md
- [X] T032 [US2] Implement mode transition in src/app.rs: enter_edit_mode() method to switch View → Edit per data-model.md
- [X] T033 [US2] Implement terminal setup in src/main.rs: enable_raw_mode, EnterAlternateScreen, setup panic hook per research.md
- [X] T034 [US2] Create keyboard event handler in src/app.rs using crossterm polling with 100ms timeout per research.md
- [X] T035 [US2] Implement view mode keybindings in src/app.rs: i/Enter for edit, q/Esc for quit per contracts/cli-interface.md
- [X] T036 [US2] Implement edit mode keybindings in src/app.rs: Esc for save, Ctrl+C for cancel, pass other keys to textarea per contracts/cli-interface.md
- [X] T037 [US2] Create save logic in src/app.rs: create backup, validate no concurrent mods, atomic write per research.md
- [X] T038 [US2] Implement backup creation in src/core/backup.rs: timestamped .gitleaksignore.backup.{timestamp} format per research.md
- [X] T039 [US2] Implement backup cleanup in src/core/backup.rs: keep last 5 backups, auto-delete old ones per research.md
- [X] T040 [US2] Add save confirmation in src/ui/viewer.rs: display updated line after successful save per spec.md
- [X] T041 [US2] Implement cancel edit in src/app.rs: discard changes, return to view mode per data-model.md
- [X] T042 [US2] Add concurrent modification check in src/app.rs: compare mtime before save per research.md
- [X] T043 [US2] Create conflict resolution warning in src/app.rs with external modification detection per contracts/cli-interface.md

**Checkpoint**: Users can now edit lines and save changes safely - US1 + US2 complete

---

## Phase 5: User Story 3 - Navigate and Browse File (Priority: P3)

**Goal**: Enable scrolling and navigation through the file without re-launching

**Independent Test**: Launch viewer, use j/k to scroll, use g/G to jump, verify smooth navigation

**Acceptance Criteria**:
- Scroll up/down by line
- Page up/down by viewport
- Jump to top/bottom
- Jump to specific line number
- Handle end-of-file boundaries gracefully

### Implementation for User Story 3

- [X] T044 [P] [US3] Implement scroll_up() method in src/app.rs: decrease scroll_offset with bounds check per data-model.md
- [X] T045 [P] [US3] Implement scroll_down() method in src/app.rs: increase scroll_offset with max limit per data-model.md
- [X] T046 [P] [US3] Implement page_up() method in src/app.rs: scroll by viewport_height lines per contracts/cli-interface.md
- [X] T047 [P] [US3] Implement page_down() method in src/app.rs: scroll by viewport_height lines per contracts/cli-interface.md
- [X] T048 [US3] Implement jump_to_top() method in src/app.rs: set scroll_offset to 0 per data-model.md
- [X] T049 [US3] Implement jump_to_bottom() method in src/app.rs: set scroll_offset to max per data-model.md
- [X] T050 [US3] Implement jump_to_line() method in src/app.rs: validate and center line number per data-model.md
- [X] T051 [US3] Add navigation keybindings in src/app.rs: j/↓ (down), k/↑ (up), d/PageDown, u/PageUp, g/Home (top), G/End (bottom) per contracts/cli-interface.md
- [X] T052 [US3] Implement line jump input in src/app.rs: :N format to jump to specific line per contracts/cli-interface.md
- [X] T053 [US3] Add viewport calculation in src/ui/viewer.rs: calculate visible_start and visible_end based on scroll_offset per research.md
- [X] T054 [US3] Implement end-of-file indicator in src/ui/viewer.rs: show message when at bottom per spec.md
- [X] T055 [US3] Update status line in src/ui/viewer.rs: show current line / total lines per contracts/cli-interface.md

**Checkpoint**: Full navigation capabilities - US1 + US2 + US3 complete

---

## Phase 6: User Story 4 - Syntax Highlighting and Pattern Recognition (Priority: P4)

**Goal**: Enhance visual distinction of pattern types for better comprehension

**Independent Test**: View file with various patterns, verify comments/hashes/paths/rules have distinct colors

**Acceptance Criteria**:
- Comments visually distinguished (dark gray + italic)
- Fingerprint components color-coded (hash=yellow, path=cyan, rule=magenta, line#=green)
- Invalid patterns highlighted (red + underlined)
- Visual distinction achievable within 3 seconds

### Implementation for User Story 4

- [ ] T056 [P] [US4] Enhance PatternType enum in src/models/pattern.rs: add detailed Fingerprint variant with commit_hash, file_path, rule_id, line_number_in_file fields per data-model.md
- [ ] T057 [US4] Implement detailed fingerprint parsing in src/models/pattern.rs: split on ':', validate commit hash (40 hex chars), extract components per research.md
- [ ] T058 [US4] Create SyntaxHighlighter in src/ui/styles.rs with highlight_line() method per research.md
- [ ] T059 [US4] Implement comment styling in src/ui/styles.rs: Style::default().fg(Color::DarkGray).italic() per research.md
- [ ] T060 [US4] Implement fingerprint component styling in src/ui/styles.rs: separate Spans for hash (yellow+bold), path (cyan), rule (magenta), line# (green) per research.md
- [ ] T061 [US4] Implement invalid pattern styling in src/ui/styles.rs: Style::default().fg(Color::Red).underlined() per research.md
- [ ] T062 [US4] Implement blank line styling in src/ui/styles.rs: default styling (invisible) per research.md
- [ ] T063 [US4] Update viewport rendering in src/ui/viewer.rs: use SyntaxHighlighter for each visible line per research.md
- [ ] T064 [US4] Implement viewport-based lazy styling in src/ui/viewer.rs: only style visible 50 lines, not entire file per research.md
- [ ] T065 [US4] Add NO_COLOR environment variable support in src/ui/styles.rs: disable syntax highlighting if set per contracts/cli-interface.md

**Checkpoint**: Full syntax highlighting - all 4 user stories complete

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Final improvements, documentation, and polish

- [X] T066 [P] Add --help flag implementation in src/main.rs with full usage information per contracts/cli-interface.md
- [X] T067 [P] Add --version flag implementation in src/main.rs displaying gli-editor 1.0.0 per contracts/cli-interface.md
- [X] T068 [P] Add --read-only flag implementation in src/main.rs to disable edit mode per contracts/cli-interface.md
- [ ] T069 [P] Implement help modal in src/ui/viewer.rs: display keybinding reference on ?/F1 per contracts/cli-interface.md
- [ ] T070 [P] Add exit code handling in src/main.rs: 0=success, 1=file error, 2=invalid args, 3=write error per contracts/cli-interface.md
- [ ] T071 Create comprehensive error messages in src/error.rs with suggestions per contracts/cli-interface.md (FileNotFound: suggest touch, PermissionDenied: suggest ls -l)
- [ ] T072 [P] Add file size limit check in src/core/file_reader.rs: warn if > 10,000 lines per spec.md assumptions
- [ ] T073 [P] Implement UTF-8 validation in src/core/file_reader.rs: reject non-UTF-8 files with clear error per data-model.md
- [ ] T074 [P] Add null byte detection in src/core/file_reader.rs: reject binary files per data-model.md
- [ ] T075 [P] Optimize viewport rendering in src/ui/viewer.rs: cache PatternType parsing results per research.md
- [ ] T076 Add integration test fixtures in tests/integration/fixtures/: sample .gitleaksignore files (empty, single-line, 100 lines, 10k lines, mixed line endings)
- [ ] T077 Add unit tests in tests/unit/pattern_tests.rs: test PatternType::parse() for all pattern types
- [ ] T078 [P] Add unit tests in tests/unit/file_reader_tests.rs: test line ending detection and preservation
- [ ] T079 [P] Add unit tests in tests/unit/backup_tests.rs: test backup creation and cleanup
- [ ] T080 Add integration test in tests/integration/view_tests.rs: test view workflow (load file, display lines, verify output)
- [ ] T081 Add integration test in tests/integration/edit_tests.rs: test edit workflow (view, edit, save, verify file updated)
- [ ] T082 Add integration test in tests/integration/navigation_tests.rs: test navigation (scroll, jump, boundaries)
- [X] T083 [P] Create user documentation in README.md: installation, usage examples, keybindings per quickstart.md
- [ ] T084 [P] Add inline documentation in all src/ modules: module-level docs, public API docs
- [X] T085 Run cargo fmt to format all code
- [X] T086 Run cargo clippy --all-targets to check for lints and warnings
- [X] T087 Build release binary with cargo build --release and verify <1s startup time per success criteria SC-008
- [ ] T088 Performance test with 10,000-line file: verify <2s load time per success criteria SC-001 and SC-004
- [ ] T089 Test edit workflow timing: verify single-line edit completes in <30s per success criteria SC-002

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup (Phase 1) completion - BLOCKS all user stories
- **User Story 1 (Phase 3)**: Depends on Foundational (Phase 2) completion
- **User Story 2 (Phase 4)**: Depends on Foundational (Phase 2) completion + US1 for view mode structure
- **User Story 3 (Phase 5)**: Depends on Foundational (Phase 2) completion + US1 for view mode
- **User Story 4 (Phase 6)**: Depends on Foundational (Phase 2) completion + US1 for rendering pipeline
- **Polish (Phase 7)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P2)**: Can start after US1 (needs view mode structure) - Independently testable for editing
- **User Story 3 (P3)**: Can start after US1 (needs view mode) - Independently testable for navigation
- **User Story 4 (P4)**: Can start after US1 (needs rendering pipeline) - Independently testable for syntax highlighting

### Within Each Phase

**Setup (Phase 1)**:
- T001 must complete first (create project)
- T002-T006 can run in parallel after T001

**Foundational (Phase 2)**:
- T007 first (error types used everywhere)
- T008-T011 in parallel (models)
- T012-T013 (file reading)
- T014-T016 in parallel (backup and safety)

**User Story 1 (Phase 3)**:
- T017-T019 in parallel (models and CLI parsing)
- T020 (view state)
- T021-T023 in parallel (UI rendering)
- T024-T026 in parallel (validation and errors)
- T027 last (wire everything together)

**User Story 2 (Phase 4)**:
- T028-T030 in parallel (edit models)
- T031-T033 (UI setup)
- T034-T036 (event handling)
- T037-T043 (save logic and safety)

**User Story 3 (Phase 5)**:
- T044-T047 in parallel (scroll methods)
- T048-T050 in parallel (jump methods)
- T051-T055 (keybindings and UI updates)

**User Story 4 (Phase 6)**:
- T056-T057 (enhanced parsing)
- T058-T062 in parallel (styling)
- T063-T065 (rendering integration)

**Polish (Phase 7)**:
- T066-T070 in parallel (CLI enhancements)
- T071-T075 in parallel (error handling and optimization)
- T076-T082 (tests - can run in parallel)
- T083-T084 in parallel (documentation)
- T085-T089 (validation and verification)

### Parallel Opportunities

**Setup Phase**: T002, T003, T004, T005, T006 can run together
**Foundational Phase**: T008-T009 together, T014-T015 together
**User Story 1**: T017-T018-T019 together, T021-T022-T023 together, T024-T025-T026 together
**User Story 2**: T028-T029-T030 together, T038-T039 together
**User Story 3**: T044-T045-T046-T047 together, T048-T049-T050 together
**User Story 4**: T059-T060-T061-T062 together
**Polish**: Most tasks marked [P] can run in parallel

---

## Parallel Example: User Story 1

```bash
# Launch model and CLI tasks together:
Task T017: "Create LineRange struct in src/models/line.rs"
Task T018: "Implement CLI argument parsing in src/main.rs"
Task T019: "Implement line specification parser in src/main.rs"

# Launch UI rendering tasks together:
Task T021: "Implement syntax highlighting logic in src/ui/styles.rs"
Task T022: "Create viewport rendering function in src/ui/viewer.rs"
Task T023: "Implement line number formatting in src/ui/viewer.rs"

# Launch validation tasks together:
Task T024: "Add file validation in src/core/file_reader.rs"
Task T025: "Implement line bounds validation in src/core/file_reader.rs"
Task T026: "Create error message formatting in src/error.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T006)
2. Complete Phase 2: Foundational (T007-T016) - CRITICAL blocking phase
3. Complete Phase 3: User Story 1 (T017-T027)
4. **STOP and VALIDATE**: Test US1 independently
   - Can view single lines with context?
   - Can view line ranges?
   - Line numbers displayed correctly?
   - Syntax highlighting working?
   - Out-of-bounds errors clear?
5. Deploy/demo if ready

**MVP Checkpoint**: With just US1, users have a functional .gitleaksignore viewer with syntax highlighting

### Incremental Delivery

1. **Setup + Foundational** (T001-T016) → Foundation ready
2. **Add US1** (T017-T027) → Test independently → **Deploy/Demo (MVP!)**
   - Value: Quick file inspection with syntax highlighting
3. **Add US2** (T028-T043) → Test independently → Deploy/Demo
   - Value: MVP + inline editing capability
4. **Add US3** (T044-T055) → Test independently → Deploy/Demo
   - Value: MVP + editing + smooth navigation
5. **Add US4** (T056-T065) → Test independently → Deploy/Demo
   - Value: Full-featured editor with enhanced visuals
6. **Polish** (T066-T089) → Final release

Each story adds value without breaking previous stories.

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup (Phase 1) together
2. Team completes Foundational (Phase 2) together - CRITICAL
3. Once Foundational is done:
   - **Developer A**: User Story 1 (T017-T027)
   - **Developer B**: Can prepare US2 models (T028-T030) but wait for US1 view mode
   - **Developer C**: Can prepare US3 navigation methods (T044-T050) but wait for US1 view mode
4. After US1 complete:
   - **Developer A**: Polish and testing (Phase 7)
   - **Developer B**: User Story 2 (T031-T043)
   - **Developer C**: User Story 3 (T051-T055)
   - **Developer D**: User Story 4 (T056-T065)
5. Stories integrate and test independently

---

## Notes

- [P] tasks = different files, no dependencies - safe to parallelize
- [Story] label maps task to specific user story for traceability and independent testing
- Each user story should be independently completable and testable
- Rust's cargo test runs tests automatically - no separate test phase needed
- Commit after each task or logical group (e.g., after each model, after each service)
- Stop at any checkpoint to validate story independently before proceeding
- Follow TDD approach from quickstart.md: write tests, see them fail, implement, see them pass
- Use cargo watch -x test for continuous testing during development
- Run cargo fmt and cargo clippy regularly to maintain code quality

---

**Total Tasks**: 89
**Task Breakdown by Phase**:
- Phase 1 (Setup): 6 tasks
- Phase 2 (Foundational): 10 tasks
- Phase 3 (US1 - View): 11 tasks
- Phase 4 (US2 - Edit): 16 tasks
- Phase 5 (US3 - Navigate): 12 tasks
- Phase 6 (US4 - Syntax): 10 tasks
- Phase 7 (Polish): 24 tasks

**Parallel Opportunities Identified**: 43 tasks marked [P] can be executed in parallel within their phase

**Independent Test Criteria**:
- US1: Launch with line spec, verify display matches file
- US2: Edit line, verify save creates backup and updates file
- US3: Navigate with j/k/g/G, verify scrolling and jumping work
- US4: View patterns, verify colors distinguish types

**Suggested MVP Scope**: Phase 1 + Phase 2 + Phase 3 (US1 only) = 27 tasks for functional viewer with syntax highlighting
