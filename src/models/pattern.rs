/// Pattern type classification for .gitleaksignore entries

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatternType {
    /// Line starting with # (comment)
    Comment,
    /// Valid gitleaks fingerprint with 3-4 components
    Fingerprint {
        commit_hash: Option<String>,
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
        // Format: [commit_hash:]file_path:rule_id:line_number
        // Note: file_path can contain ':' for archives (e.g., archive.tar.gz:inner.tar:file.env)

        // Find the last ':' for line_number
        let Some(last_colon) = trimmed.rfind(':') else {
            return PatternType::Invalid;
        };

        let line_number_str = &trimmed[last_colon + 1..];
        let Ok(line_number) = line_number_str.parse::<u32>() else {
            return PatternType::Invalid;
        };

        let rest = &trimmed[..last_colon];

        // Find second-to-last ':' for rule_id
        let Some(second_last_colon) = rest.rfind(':') else {
            return PatternType::Invalid;
        };

        let rule_id = &rest[second_last_colon + 1..];
        let remaining = &rest[..second_last_colon];

        // Check if there's a third ':' for commit_hash
        if let Some(third_last_colon) = remaining.rfind(':') {
            let potential_hash = &remaining[..third_last_colon];

            // Check if it looks like a commit hash (40 hex chars)
            if potential_hash.len() == 40 && potential_hash.chars().all(|c| c.is_ascii_hexdigit()) {
                let file_path = &remaining[third_last_colon + 1..];

                if !file_path.is_empty() && !rule_id.is_empty() {
                    return PatternType::Fingerprint {
                        commit_hash: Some(potential_hash.to_string()),
                        file_path: file_path.to_string(),
                        rule_id: rule_id.to_string(),
                        line_number,
                    };
                }
            }
        }

        // No commit hash, treat remaining as file_path
        let file_path = remaining;

        // Validate that we have non-empty components
        if file_path.is_empty() || rule_id.is_empty() {
            return PatternType::Invalid;
        }

        PatternType::Fingerprint {
            commit_hash: None,
            file_path: file_path.to_string(),
            rule_id: rule_id.to_string(),
            line_number,
        }
    }
}
