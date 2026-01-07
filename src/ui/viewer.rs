/// Line viewing widget
use crate::app::{EditState, ViewState};
use crate::models::pattern::PatternType;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
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

        // Render content area
        Self::render_content(f, view_state, chunks[0]);

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

    fn render_status(
        f: &mut Frame,
        view_state: &ViewState,
        save_message: Option<&str>,
        area: Rect,
    ) {
        let status = if let Some(msg) = save_message {
            format!(" VIEW | {} ", msg)
        } else {
            format!(
                " VIEW | Line {}/{} (showing {}-{}) | j/k:scroll u/d:page g/G:jump i:edit q:quit ",
                view_state.current_line,
                view_state.file_context.total_lines,
                view_state.visible_range.start_line,
                view_state.visible_range.end_line
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
                vec![
                    Span::styled(
                        format!("{}:", commit_hash),
                        base_style
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(format!("{}:", file_path), base_style.fg(Color::Cyan)),
                    Span::styled(format!("{}:", rule_id), base_style.fg(Color::Magenta)),
                    Span::styled(line_number.to_string(), base_style.fg(Color::Green)),
                ]
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
