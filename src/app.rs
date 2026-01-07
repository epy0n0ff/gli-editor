/// Application state and main loop for gli-editor
use crate::core::backup::BackupManager;
use crate::core::file_reader::FileContext;
use crate::error::Result;
use crate::models::line::LineRange;
use crate::ui::viewer::ViewerWidget;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::path::PathBuf;
use tui_textarea::TextArea;

/// View mode state
pub struct ViewState {
    pub file_context: FileContext,
    pub visible_range: LineRange,
    pub scroll_offset: usize,
    pub current_line: usize,
    pub preview_enabled: bool,
    pub preview_content: Option<PreviewContent>,
}

/// Preview content for the selected line
#[derive(Debug, Clone)]
pub struct PreviewContent {
    pub file_path: String,
    pub target_line: usize,
    pub lines: Vec<String>,
    pub start_line: usize,
}

impl ViewState {
    pub fn new(file_context: FileContext, start_line: usize, end_line: usize) -> Result<Self> {
        let lines = file_context.get_range(start_line, end_line)?;
        let visible_range = LineRange::new(start_line, end_line, lines);

        // Set current_line to 1 for non-empty files, 0 for empty files
        let current_line = if start_line == 0 && end_line == 0 {
            0 // Empty file
        } else {
            start_line.max(1) // Ensure at least line 1 for non-empty files
        };

        Ok(Self {
            file_context,
            visible_range,
            scroll_offset: 0,
            current_line,
            preview_enabled: true,
            preview_content: None,
        })
    }

    /// Update preview content for the current line
    pub fn update_preview(&mut self) {
        self.preview_content = None;

        if !self.preview_enabled {
            return;
        }

        // Get current line
        if let Some(line) = self.file_context.get_line(self.current_line) {
            // Extract file path and line number from fingerprint
            if let crate::models::pattern::PatternType::Fingerprint {
                file_path,
                line_number,
                ..
            } = &line.pattern_type
            {
                // Try to read the target file
                if let Ok(content) = Self::read_preview_file(file_path, *line_number) {
                    self.preview_content = Some(content);
                }
            }
        }
    }

    /// Read preview content from target file
    fn read_preview_file(file_path: &str, target_line: u32) -> Result<PreviewContent> {
        use std::fs::File;
        use std::io::{BufRead, BufReader};

        let target_line = target_line as usize;
        let context = 10; // Show ±10 lines around target

        let file = File::open(file_path).map_err(|_| {
            crate::error::GliError::FileNotFound(format!("Preview file not found: {}", file_path))
        })?;

        let reader = BufReader::new(file);
        let all_lines: Vec<String> = reader
            .lines()
            .filter_map(|l| l.ok())
            .collect();

        // Validate target_line is within file bounds
        if all_lines.is_empty() {
            return Err(crate::error::GliError::FileNotFound(
                "Preview file is empty".to_string(),
            ));
        }

        // Clamp target_line to file bounds
        let target_line = target_line.min(all_lines.len());

        let start_line = target_line.saturating_sub(context).max(1);
        let end_line = (target_line + context).min(all_lines.len());

        // Ensure start_line is valid for slicing
        let start_idx = (start_line - 1).min(all_lines.len().saturating_sub(1));
        let end_idx = end_line.min(all_lines.len());

        let lines = if start_idx < end_idx {
            all_lines[start_idx..end_idx].to_vec()
        } else {
            vec![]
        };

        Ok(PreviewContent {
            file_path: file_path.to_string(),
            target_line,
            lines,
            start_line,
        })
    }
}

/// Edit mode state
pub struct EditState {
    pub textarea: TextArea<'static>,
    pub original_line: usize,
    pub original_content: String,
}

impl EditState {
    pub fn new(line_number: usize, content: String) -> Self {
        let mut textarea = TextArea::default();
        textarea.insert_str(&content);
        textarea.move_cursor(tui_textarea::CursorMove::End);

        Self {
            textarea,
            original_line: line_number,
            original_content: content,
        }
    }

    pub fn has_changes(&self) -> bool {
        let current = self.textarea.lines().join("");
        current != self.original_content
    }

    pub fn get_content(&self) -> String {
        self.textarea.lines().join("")
    }
}

/// Application mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    View,
    Edit,
}

pub struct App {
    mode: AppMode,
    view_state: ViewState,
    edit_state: Option<EditState>,
    read_only: bool,
    should_quit: bool,
    backup_manager: BackupManager,
    save_message: Option<String>,
}

impl App {
    pub fn new(file_path: PathBuf, line_spec: crate::LineSpec, read_only: bool) -> Result<Self> {
        // Load file
        let file_context = FileContext::load(file_path)?;

        // Calculate display range from line specification
        let (start_line, end_line) = line_spec.calculate_range(file_context.total_lines)?;

        // Create view state
        let mut view_state = ViewState::new(file_context, start_line, end_line)?;

        // Initialize preview for the first line
        view_state.update_preview();

        Ok(Self {
            mode: AppMode::View,
            view_state,
            edit_state: None,
            read_only,
            should_quit: false,
            backup_manager: BackupManager::new(),
            save_message: None,
        })
    }

    /// Enter edit mode for the current line (T032)
    fn enter_edit_mode(&mut self) -> Result<()> {
        if self.read_only {
            self.save_message = Some("Read-only mode: editing disabled".to_string());
            return Ok(());
        }

        let line_number = self.view_state.current_line;
        if let Some(line) = self.view_state.file_context.get_line(line_number) {
            let edit_state = EditState::new(line_number, line.content.clone());
            self.edit_state = Some(edit_state);
            self.mode = AppMode::Edit;
            self.save_message = None;
        }

        Ok(())
    }

    /// Save edit and return to view mode (T037)
    fn save_edit(&mut self) -> Result<()> {
        if let Some(edit_state) = &self.edit_state {
            if !edit_state.has_changes() {
                self.mode = AppMode::View;
                self.edit_state = None;
                return Ok(());
            }

            let line_number = edit_state.original_line;
            let new_content = edit_state.get_content();

            // Create backup (T038)
            let backup_path = self
                .backup_manager
                .create_backup(&self.view_state.file_context.file_path)?;

            // Check for concurrent modifications (T042)
            if self
                .view_state
                .file_context
                .check_for_external_modifications()?
            {
                self.save_message = Some("Warning: File modified externally!".to_string());
                // In a full implementation, show conflict resolution modal (T043)
                // For now, we'll just warn and proceed
            }

            // Update the line in file context
            self.view_state
                .file_context
                .update_line(line_number, new_content.clone())?;

            // Write atomically (T015 already implemented)
            self.view_state.file_context.write_atomic()?;

            // Update visible range
            let (start, end) = (
                self.view_state.visible_range.start_line,
                self.view_state.visible_range.end_line,
            );
            let lines = self.view_state.file_context.get_range(start, end)?;
            self.view_state.visible_range = LineRange::new(start, end, lines);

            self.save_message = Some(format!(
                "Saved line {} (backup: {})",
                line_number,
                backup_path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("created")
            ));
        }

        self.mode = AppMode::View;
        self.edit_state = None;
        Ok(())
    }

    /// Cancel edit and return to view mode (T041)
    fn cancel_edit(&mut self) {
        self.mode = AppMode::View;
        self.edit_state = None;
        self.save_message = Some("Edit cancelled".to_string());
    }

    /// Scroll up by one line (T044)
    fn scroll_up(&mut self) -> Result<()> {
        // Move cursor up if not at top of file
        if self.view_state.current_line > 1 {
            self.view_state.current_line -= 1;

            // Calculate scroll margin (lines from top/bottom before scrolling)
            let scroll_margin = 3;
            let page_size = self.view_state.visible_range.end_line - self.view_state.visible_range.start_line;

            // Scroll viewport if cursor approaches top edge
            let distance_from_top = self.view_state.current_line.saturating_sub(self.view_state.visible_range.start_line);
            if distance_from_top < scroll_margin && self.view_state.visible_range.start_line > 1 {
                // Scroll viewport up by one line
                let new_start = self.view_state.visible_range.start_line.saturating_sub(1);
                let new_end = (new_start + page_size).min(self.view_state.file_context.total_lines);
                self.update_visible_range(new_start, new_end)?;
            }

            // Update preview for new line
            self.view_state.update_preview();
        }
        Ok(())
    }

    /// Scroll down by one line (T045)
    fn scroll_down(&mut self) -> Result<()> {
        // Move cursor down if not at end of file
        if self.view_state.current_line < self.view_state.file_context.total_lines {
            self.view_state.current_line += 1;

            // Calculate scroll margin (lines from top/bottom before scrolling)
            let scroll_margin = 3;
            let page_size = self.view_state.visible_range.end_line - self.view_state.visible_range.start_line;

            // Scroll viewport if cursor approaches bottom edge
            let distance_from_bottom = self.view_state.visible_range.end_line.saturating_sub(self.view_state.current_line);
            if distance_from_bottom < scroll_margin && self.view_state.visible_range.end_line < self.view_state.file_context.total_lines {
                // Scroll viewport down by one line
                let new_start = self.view_state.visible_range.start_line + 1;
                let new_end = (new_start + page_size).min(self.view_state.file_context.total_lines);
                self.update_visible_range(new_start, new_end)?;
            }

            // Update preview for new line
            self.view_state.update_preview();
        }
        Ok(())
    }

    /// Scroll up by page (T046)
    fn page_up(&mut self) -> Result<()> {
        let page_size =
            self.view_state.visible_range.end_line - self.view_state.visible_range.start_line;
        let new_start = self
            .view_state
            .visible_range
            .start_line
            .saturating_sub(page_size)
            .max(1);
        let new_end = (new_start + page_size).min(self.view_state.file_context.total_lines);
        self.update_visible_range(new_start, new_end)?;
        self.view_state.current_line = new_start;
        Ok(())
    }

    /// Scroll down by page (T047)
    fn page_down(&mut self) -> Result<()> {
        let page_size =
            self.view_state.visible_range.end_line - self.view_state.visible_range.start_line;

        // Calculate new_start, but ensure we don't go beyond what allows a valid range
        let total_lines = self.view_state.file_context.total_lines;
        let new_start = (self.view_state.visible_range.start_line + page_size)
            .min(total_lines.saturating_sub(page_size).max(1));
        let new_end = (new_start + page_size).min(total_lines);

        self.update_visible_range(new_start, new_end)?;
        self.view_state.current_line = new_start;
        Ok(())
    }

    /// Jump to top of file (T048)
    fn jump_to_top(&mut self) -> Result<()> {
        let page_size =
            self.view_state.visible_range.end_line - self.view_state.visible_range.start_line;
        let new_end = (1 + page_size).min(self.view_state.file_context.total_lines);
        self.update_visible_range(1, new_end)?;
        self.view_state.current_line = 1;
        Ok(())
    }

    /// Jump to bottom of file (T049)
    fn jump_to_bottom(&mut self) -> Result<()> {
        let page_size =
            self.view_state.visible_range.end_line - self.view_state.visible_range.start_line;
        let new_start = (self
            .view_state
            .file_context
            .total_lines
            .saturating_sub(page_size - 1))
        .max(1);
        let new_end = self.view_state.file_context.total_lines;
        self.update_visible_range(new_start, new_end)?;
        self.view_state.current_line = new_end;
        Ok(())
    }

    /// Jump to specific line (T050)
    fn jump_to_line(&mut self, target_line: usize) -> Result<()> {
        if target_line == 0 || target_line > self.view_state.file_context.total_lines {
            self.save_message = Some(format!("Invalid line number: {}", target_line));
            return Ok(());
        }

        let page_size =
            self.view_state.visible_range.end_line - self.view_state.visible_range.start_line;
        let half_page = page_size / 2;

        let new_start = target_line.saturating_sub(half_page).max(1);
        let new_end = (new_start + page_size).min(self.view_state.file_context.total_lines);

        self.update_visible_range(new_start, new_end)?;
        self.view_state.current_line = target_line;
        self.save_message = Some(format!("Jumped to line {}", target_line));
        Ok(())
    }

    /// Update visible range helper (T053)
    fn update_visible_range(&mut self, start: usize, end: usize) -> Result<()> {
        let lines = self.view_state.file_context.get_range(start, end)?;
        self.view_state.visible_range = LineRange::new(start, end, lines);
        self.view_state.scroll_offset = start - 1;
        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Adjust viewport to terminal size
        self.adjust_viewport_to_screen(&mut terminal)?;

        // Main loop
        let result = self.run_loop(&mut terminal);

        // Cleanup terminal
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        result
    }

    /// Adjust viewport to fit terminal screen size
    fn adjust_viewport_to_screen<B: ratatui::backend::Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
    ) -> Result<()> {
        let size = terminal.size()?;

        // Calculate available height for content (minus status line and borders)
        let content_height = size.height.saturating_sub(3) as usize; // -1 status, -2 borders

        if content_height == 0 {
            return Ok(());
        }

        // Adjust visible range to screen size
        let current_line = self.view_state.current_line;
        let total_lines = self.view_state.file_context.total_lines;

        // Center current line in viewport
        let half_height = content_height / 2;
        let new_start = current_line.saturating_sub(half_height).max(1);
        let new_end = (new_start + content_height - 1).min(total_lines);

        // Update visible range
        let lines = self.view_state.file_context.get_range(new_start, new_end)?;
        self.view_state.visible_range = LineRange::new(new_start, new_end, lines);
        self.view_state.scroll_offset = new_start - 1;

        Ok(())
    }

    fn run_loop<B: ratatui::backend::Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {
            // Render UI based on mode
            terminal.draw(|f| match self.mode {
                AppMode::View => {
                    ViewerWidget::render(f, &self.view_state, self.save_message.as_deref());
                }
                AppMode::Edit => {
                    if let Some(ref mut edit_state) = self.edit_state {
                        ViewerWidget::render_edit_mode(f, &self.view_state, edit_state);
                    }
                }
            })?;

            // Handle input with 100ms polling (T034)
            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    self.handle_key(key)?;
                }
            }

            // Check if we should quit
            if self.should_quit {
                break;
            }
        }

        Ok(())
    }

    /// Handle keyboard events based on current mode (T035, T036)
    fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        match self.mode {
            AppMode::View => {
                // View mode keybindings (T035, T051)
                match key.code {
                    KeyCode::Char('q') => {
                        self.should_quit = true;
                    }
                    KeyCode::Char('i') | KeyCode::Enter => {
                        self.enter_edit_mode()?;
                    }
                    KeyCode::Esc => {
                        // Clear save message on Esc in view mode
                        self.save_message = None;
                    }
                    // Navigation: scroll up
                    KeyCode::Char('k') | KeyCode::Up => {
                        self.scroll_up()?;
                    }
                    // Navigation: scroll down
                    KeyCode::Char('j') | KeyCode::Down => {
                        self.scroll_down()?;
                    }
                    // Navigation: page up
                    KeyCode::Char('u') | KeyCode::PageUp => {
                        self.page_up()?;
                    }
                    // Navigation: page down
                    KeyCode::Char('d') | KeyCode::PageDown => {
                        self.page_down()?;
                    }
                    // Navigation: jump to top
                    KeyCode::Char('g') | KeyCode::Home => {
                        self.jump_to_top()?;
                    }
                    // Navigation: jump to bottom
                    KeyCode::Char('G') | KeyCode::End => {
                        self.jump_to_bottom()?;
                    }
                    // Toggle preview pane
                    KeyCode::Char('p') => {
                        self.view_state.preview_enabled = !self.view_state.preview_enabled;
                        if self.view_state.preview_enabled {
                            self.view_state.update_preview();
                        }
                    }
                    _ => {}
                }
            }
            AppMode::Edit => {
                // Edit mode keybindings (T036)
                match key.code {
                    KeyCode::Esc => {
                        // Save and exit edit mode
                        self.save_edit()?;
                    }
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        // Cancel edit
                        self.cancel_edit();
                    }
                    _ => {
                        // Pass all other keys to textarea
                        if let Some(ref mut edit_state) = self.edit_state {
                            edit_state.textarea.input(key);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
