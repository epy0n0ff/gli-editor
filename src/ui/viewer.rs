/// Line viewing widget
use crate::app::{EditState, PreviewContent, ViewState};
use crate::models::pattern::PatternType;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub struct ViewerWidget;

impl ViewerWidget {
    /// Render the viewer widget in view mode
    pub fn render(f: &mut Frame, view_state: &ViewState, save_message: Option<&str>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(f.size());

        // Split content area into left (gitleaksignore) and right (preview) if preview is enabled
        if view_state.preview_enabled && view_state.preview_content.is_some() {
            let content_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(chunks[0]);

            // Render gitleaksignore content on left
            Self::render_content(f, view_state, content_chunks[0]);

            // Render preview on right
            if let Some(ref preview) = view_state.preview_content {
                Self::render_preview(f, preview, content_chunks[1]);
            }
        } else {
            // Render full-width content
            Self::render_content(f, view_state, chunks[0]);
        }

        // Render status line with optional save message (T040)
        Self::render_status(f, view_state, save_message, chunks[1]);
    }

    /// Render the viewer widget in edit mode (T031)
    pub fn render_edit_mode(f: &mut Frame, view_state: &ViewState, edit_state: &mut EditState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),
                Constraint::Length(3),
                Constraint::Length(1),
            ])
            .split(f.size());

        // Render content area (same as view mode)
        Self::render_content(f, view_state, chunks[0]);

        // Render edit area - create a bordered block for the textarea
        let content = edit_state.textarea.lines().join("");
        let cursor_pos = edit_state.textarea.cursor();

        let edit_line = Line::from(vec![
            Span::raw(&content[..cursor_pos.1]),
            Span::styled("█", Style::default().fg(Color::White)),
            Span::raw(&content[cursor_pos.1..]),
        ]);

        let edit_block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" Editing line {} ", edit_state.original_line))
            .style(Style::default().fg(Color::Yellow));

        let edit_paragraph = Paragraph::new(vec![edit_line]).block(edit_block);
        f.render_widget(edit_paragraph, chunks[1]);

        // Render edit mode status
        let status = " EDIT | Esc:save  Ctrl+C:cancel  ←/→:move cursor ";
        let paragraph =
            Paragraph::new(status).style(Style::default().bg(Color::Yellow).fg(Color::Black));
        f.render_widget(paragraph, chunks[2]);
    }

    fn render_content(f: &mut Frame, view_state: &ViewState, area: Rect) {
        let mut lines = Vec::new();

        for line in &view_state.visible_range.entries {
            let is_current = line.line_number == view_state.current_line;

            // Add cursor indicator for current line
            let cursor_indicator = if is_current { ">" } else { " " };
            let line_number_str = format!("{}{:>4} ", cursor_indicator, line.line_number);

            let line_number_style = if is_current {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let mut spans = vec![Span::styled(line_number_str, line_number_style)];

            // Add syntax-highlighted content with background highlight for current line
            let content_spans = Self::highlight_line(&line.content, &line.pattern_type, is_current);
            spans.extend(content_spans);

            lines.push(Line::from(spans));
        }

        let paragraph =
            Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(format!(
                        " {} ",
                        view_state
                            .file_context
                            .file_path
                            .file_name()
                            .and_then(|s| s.to_str())
                            .unwrap_or(".gitleaksignore")
                    )));

        f.render_widget(paragraph, area);
    }

    fn render_preview(f: &mut Frame, preview: &PreviewContent, area: Rect) {
        let mut lines = Vec::new();

        for (idx, line_content) in preview.lines.iter().enumerate() {
            let line_num = preview.start_line + idx;
            let is_target = line_num == preview.target_line;

            let line_number_str = format!("{:>4} ", line_num);
            let line_number_style = if is_target {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let content_style = if is_target {
                Style::default().bg(Color::Rgb(60, 40, 40)).fg(Color::White)
            } else {
                Style::default()
            };

            let mut spans = vec![Span::styled(line_number_str, line_number_style)];
            spans.push(Span::styled(line_content.clone(), content_style));

            lines.push(Line::from(spans));
        }

        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" Preview: {} (line {}) ", preview.file_path, preview.target_line))
            .style(Style::default().fg(Color::Cyan));

        let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: false });

        f.render_widget(paragraph, area);
    }

    fn render_status(
        f: &mut Frame,
        view_state: &ViewState,
        save_message: Option<&str>,
        area: Rect,
    ) {
        let preview_status = if view_state.preview_enabled { "p:toggle" } else { "p:enable" };

        let status = if let Some(msg) = save_message {
            format!(" VIEW | {} ", msg)
        } else {
            format!(
                " VIEW | Line {}/{} (showing {}-{}) | j/k:scroll {} i:edit q:quit ",
                view_state.current_line,
                view_state.file_context.total_lines,
                view_state.visible_range.start_line,
                view_state.visible_range.end_line,
                preview_status
            )
        };

        let paragraph =
            Paragraph::new(status).style(Style::default().bg(Color::DarkGray).fg(Color::White));

        f.render_widget(paragraph, area);
    }

    /// Apply syntax highlighting to a line based on its pattern type
    fn highlight_line(content: &str, pattern_type: &PatternType, is_current: bool) -> Vec<Span<'static>> {
        let base_style = if is_current {
            Style::default().bg(Color::Rgb(40, 40, 50))
        } else {
            Style::default()
        };

        match pattern_type {
            PatternType::Comment => {
                vec![Span::styled(
                    content.to_string(),
                    base_style
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::ITALIC),
                )]
            }
            PatternType::Fingerprint {
                commit_hash,
                file_path,
                rule_id,
                line_number,
            } => {
                let mut spans = Vec::new();

                // Add commit hash if present
                if let Some(hash) = commit_hash {
                    spans.push(Span::styled(
                        format!("{}:", hash),
                        base_style
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ));
                }

                // Add file path, rule ID, and line number
                spans.push(Span::styled(format!("{}:", file_path), base_style.fg(Color::Cyan)));
                spans.push(Span::styled(format!("{}:", rule_id), base_style.fg(Color::Magenta)));
                spans.push(Span::styled(line_number.to_string(), base_style.fg(Color::Green)));

                spans
            }
            PatternType::BlankLine => {
                vec![Span::styled(content.to_string(), base_style)]
            }
            PatternType::Invalid => {
                vec![Span::styled(
                    content.to_string(),
                    base_style
                        .fg(Color::Red)
                        .add_modifier(Modifier::UNDERLINED),
                )]
            }
        }
    }
}
