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
}
