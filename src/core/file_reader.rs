/// File reading operations
use crate::error::{GliError, Result};
use crate::models::line::Line;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tempfile::NamedTempFile;

/// Line ending format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineEnding {
    /// Unix-style \n
    LF,
    /// Windows-style \r\n
    CRLF,
    /// Legacy Mac \r
    CR,
}

impl LineEnding {
    /// Detect line ending format from file content
    ///
    /// Reads first few lines to determine the line ending format.
    /// Falls back to LF if no line endings are found.
    pub fn detect(file_path: &Path) -> Result<Self> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);

        // Read first few lines to detect line ending
        for line in reader.lines().take(10) {
            let _ = line?; // Just need to ensure the file is readable
        }

        // Re-open file and read raw bytes to detect line endings
        let content = fs::read_to_string(file_path)?;

        if content.contains("\r\n") {
            Ok(LineEnding::CRLF)
        } else if content.contains('\n') {
            Ok(LineEnding::LF)
        } else if content.contains('\r') {
            Ok(LineEnding::CR)
        } else {
            // Default to LF if no line endings found (empty file or single line)
            Ok(LineEnding::LF)
        }
    }

    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            LineEnding::LF => "\n",
            LineEnding::CRLF => "\r\n",
            LineEnding::CR => "\r",
        }
    }
}

/// File context with metadata
#[derive(Debug, Clone)]
pub struct FileContext {
    /// Absolute path to .gitleaksignore file
    pub file_path: PathBuf,
    /// Number of lines in file
    pub total_lines: usize,
    /// Detected line ending format
    pub line_ending_format: LineEnding,
    /// File modification timestamp
    pub last_modified_time: SystemTime,
    /// All lines in the file
    pub lines: Vec<Line>,
}

impl FileContext {
    /// Load file from path
    pub fn load<P: AsRef<Path>>(file_path: P) -> Result<Self> {
        let path = file_path.as_ref();

        // Check file exists
        if !path.exists() {
            return Err(GliError::FileNotFound(path.display().to_string()));
        }

        // Check file is readable
        let metadata = fs::metadata(path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                GliError::PermissionDenied(path.display().to_string())
            } else {
                GliError::IoError(e)
            }
        })?;

        let last_modified_time = metadata.modified()?;

        // Detect line ending format
        let line_ending_format = LineEnding::detect(path)?;

        // Read file with line ending preservation
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut lines = Vec::new();
        let mut line_number = 1;

        for line_result in reader.lines() {
            let content = line_result.map_err(|e| {
                if let Some(err_code) = e.raw_os_error() {
                    if err_code == 84 || err_code == 22 {
                        // Invalid UTF-8
                        GliError::InvalidEncoding(path.display().to_string())
                    } else {
                        GliError::IoError(e)
                    }
                } else {
                    GliError::IoError(e)
                }
            })?;

            lines.push(Line::new(line_number, content));
            line_number += 1;
        }

        let total_lines = lines.len();

        Ok(Self {
            file_path: path.to_path_buf(),
            total_lines,
            line_ending_format,
            last_modified_time,
            lines,
        })
    }

    /// Refresh metadata from filesystem
    pub fn refresh_metadata(&mut self) -> Result<()> {
        let metadata = fs::metadata(&self.file_path)?;
        self.last_modified_time = metadata.modified()?;
        Ok(())
    }

    /// Check if file has been modified externally
    pub fn check_for_external_modifications(&self) -> Result<bool> {
        let metadata = fs::metadata(&self.file_path)?;
        let current_mtime = metadata.modified()?;
        Ok(current_mtime > self.last_modified_time)
    }

    /// Get a specific line by number (1-based)
    pub fn get_line(&self, line_number: usize) -> Option<&Line> {
        if line_number == 0 || line_number > self.total_lines {
            return None;
        }
        self.lines.get(line_number - 1)
    }

    /// Get a range of lines
    pub fn get_range(&self, start: usize, end: usize) -> Result<Vec<Line>> {
        // Handle empty files
        if start == 0 && end == 0 {
            return Ok(Vec::new());
        }

        if start == 0 || end == 0 {
            return Err(GliError::InvalidArguments(
                "Line numbers must be >= 1".to_string(),
            ));
        }

        if start > self.total_lines {
            return Err(GliError::LineOutOfBounds(start, self.total_lines));
        }

        if end > self.total_lines {
            return Err(GliError::LineOutOfBounds(end, self.total_lines));
        }

        if start > end {
            return Err(GliError::InvalidArguments(format!(
                "Start line {} cannot be greater than end line {}",
                start, end
            )));
        }

        Ok(self.lines[(start - 1)..end].to_vec())
    }

    /// Write file atomically with line ending preservation
    ///
    /// Uses tempfile + rename for atomic write operation
    pub fn write_atomic(&mut self) -> Result<()> {
        let parent = self
            .file_path
            .parent()
            .ok_or_else(|| GliError::WriteFailure("Invalid file path".to_string()))?;

        // Create temporary file in the same directory
        let mut temp_file = NamedTempFile::new_in(parent)
            .map_err(|e| GliError::WriteFailure(format!("Failed to create temp file: {}", e)))?;

        // Write all lines with preserved line endings
        let line_ending = self.line_ending_format.as_str();
        for line in &self.lines {
            write!(temp_file, "{}{}", line.content, line_ending).map_err(|e| {
                GliError::WriteFailure(format!("Failed to write to temp file: {}", e))
            })?;
        }

        // Persist the temp file to the target path (atomic rename)
        temp_file
            .persist(&self.file_path)
            .map_err(|e| GliError::WriteFailure(format!("Failed to persist temp file: {}", e)))?;

        // Update metadata after successful write
        self.refresh_metadata()?;

        Ok(())
    }

    /// Update a specific line's content
    pub fn update_line(&mut self, line_number: usize, new_content: String) -> Result<()> {
        if line_number == 0 || line_number > self.total_lines {
            return Err(GliError::LineOutOfBounds(line_number, self.total_lines));
        }

        let line = &mut self.lines[line_number - 1];
        line.content = new_content.clone();
        line.pattern_type = crate::models::pattern::PatternType::parse(&new_content);

        Ok(())
    }

    /// Delete a specific line
    pub fn delete_line(&mut self, line_number: usize) -> Result<()> {
        if line_number == 0 || line_number > self.total_lines {
            return Err(GliError::LineOutOfBounds(line_number, self.total_lines));
        }

        // Remove the line from the vector
        self.lines.remove(line_number - 1);

        // Update total_lines count
        self.total_lines = self.lines.len();

        // Re-number all subsequent lines
        for (idx, line) in self.lines.iter_mut().enumerate().skip(line_number - 1) {
            line.line_number = idx + 1;
        }

        Ok(())
    }
}

/// FileSnapshot for concurrent modification detection
#[derive(Debug, Clone)]
pub struct FileSnapshot {
    pub file_path: PathBuf,
    pub last_modified_time: SystemTime,
}

impl FileSnapshot {
    /// Create a snapshot of the current file state
    pub fn capture<P: AsRef<Path>>(file_path: P) -> Result<Self> {
        let path = file_path.as_ref();
        let metadata = fs::metadata(path)?;
        let last_modified_time = metadata.modified()?;

        Ok(Self {
            file_path: path.to_path_buf(),
            last_modified_time,
        })
    }

    /// Check if the file has been modified since the snapshot
    pub fn has_changed(&self) -> Result<bool> {
        let metadata = fs::metadata(&self.file_path)?;
        let current_mtime = metadata.modified()?;
        Ok(current_mtime > self.last_modified_time)
    }
}

pub struct FileReader;

impl FileReader {
    pub fn new() -> Self {
        Self
    }

    /// Read a .gitleaksignore file
    pub fn read_file<P: AsRef<Path>>(path: P) -> Result<FileContext> {
        FileContext::load(path)
    }
}
