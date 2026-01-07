/// Line data structures
use crate::models::pattern::PatternType;

/// Represents a single line in the .gitleaksignore file
///
/// Also known as IgnorePatternEntry in data-model.md
#[derive(Debug, Clone)]
pub struct Line {
    /// 1-based line number in file
    pub line_number: usize,
    /// Raw line content including any whitespace (but excluding line ending)
    pub content: String,
    /// Classification of the line content
    pub pattern_type: PatternType,
}

impl Line {
    /// Create a new Line with parsed pattern type
    pub fn new(line_number: usize, content: String) -> Self {
        let pattern_type = PatternType::parse(&content);
        Self {
            line_number,
            content,
            pattern_type,
        }
    }
}

/// Represents a continuous sequence of lines for display or navigation
#[derive(Debug, Clone)]
pub struct LineRange {
    /// First line number (1-based, inclusive)
    pub start_line: usize,
    /// Last line number (1-based, inclusive)
    pub end_line: usize,
    /// Lines within the range
    pub entries: Vec<Line>,
}

impl LineRange {
    /// Create a new LineRange
    pub fn new(start_line: usize, end_line: usize, entries: Vec<Line>) -> Self {
        Self {
            start_line,
            end_line,
            entries,
        }
    }

    /// Get the total number of lines in this range
    pub fn total_lines(&self) -> usize {
        self.end_line - self.start_line + 1
    }

    /// Check if the range is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Check if a line number is within this range
    pub fn contains_line(&self, line_number: usize) -> bool {
        line_number >= self.start_line && line_number <= self.end_line
    }

    /// Get a line by its number (returns None if not in range)
    pub fn get_line(&self, line_number: usize) -> Option<&Line> {
        if !self.contains_line(line_number) {
            return None;
        }
        let index = line_number - self.start_line;
        self.entries.get(index)
    }
}
