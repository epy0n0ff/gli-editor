/// Pattern type classification for .gitleaksignore entries

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatternType {
    /// Line starting with # (comment)
    Comment,
    /// Valid gitleaks fingerprint with 4 components
    Fingerprint {
        commit_hash: String,
        file_path: String,
        rule_id: String,
        line_number: u32,
    },
    /// Empty or whitespace-only line
    BlankLine,
    /// Malformed entry
    Invalid,
}

impl PatternType {
    /// Parse a line to detect its pattern type
    ///
    /// Uses hand-written parser for performance (2-5x faster than regex per research.md)
    ///
    /// # Pattern Detection Rules
    ///
    /// 1. BlankLine: Empty or whitespace-only
    /// 2. Comment: Line starts with '#' (after trimming whitespace)
    /// 3. Fingerprint: Valid format with 4 components separated by ':'
    ///    - commit_hash:file_path:rule_id:line_number
    ///    - commit_hash must be exactly 40 hexadecimal characters
    ///    - line_number must be parseable as u32
    /// 4. Invalid: Anything else
    pub fn parse(line: &str) -> Self {
        let trimmed = line.trim();

        // Check for blank line
        if trimmed.is_empty() {
            return PatternType::BlankLine;
        }

        // Check for comment
        if trimmed.starts_with('#') {
            return PatternType::Comment;
        }

        // Try to parse as fingerprint
        // Format: commit_hash:file_path:rule_id:line_number
        // Note: file_path can contain ':' for archives (e.g., archive.tar.gz:inner.tar:file.env)

        // Find the first ':' - that should end the commit hash
        let Some(first_colon) = trimmed.find(':') else {
            return PatternType::Invalid;
        };

        let commit_hash = &trimmed[..first_colon];

        // Validate commit hash: exactly 40 hex characters
        if commit_hash.len() != 40 || !commit_hash.chars().all(|c| c.is_ascii_hexdigit()) {
            return PatternType::Invalid;
        }

        let rest = &trimmed[first_colon + 1..];

        // Find the last two ':' for rule_id and line_number
        // We work backwards to handle file paths with colons
        let Some(last_colon) = rest.rfind(':') else {
            return PatternType::Invalid;
        };

        let line_number_str = &rest[last_colon + 1..];
        let Ok(line_number) = line_number_str.parse::<u32>() else {
            return PatternType::Invalid;
        };

        let middle = &rest[..last_colon];
        let Some(second_last_colon) = middle.rfind(':') else {
            return PatternType::Invalid;
        };

        let rule_id = &middle[second_last_colon + 1..];
        let file_path = &middle[..second_last_colon];

        // Validate that we have non-empty components
        if file_path.is_empty() || rule_id.is_empty() {
            return PatternType::Invalid;
        }

        PatternType::Fingerprint {
            commit_hash: commit_hash.to_string(),
            file_path: file_path.to_string(),
            rule_id: rule_id.to_string(),
            line_number,
        }
    }
}
