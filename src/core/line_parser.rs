/// Pattern type detection and parsing
use crate::models::pattern::PatternType;

pub struct LineParser;

impl LineParser {
    /// Parse a line to detect its pattern type
    ///
    /// This is a thin wrapper around PatternType::parse() for convenience.
    pub fn parse(line: &str) -> PatternType {
        PatternType::parse(line)
    }
}
