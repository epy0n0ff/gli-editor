/// Error types for gli-editor
use std::fmt;

#[derive(Debug)]
pub enum GliError {
    /// File not found at specified path
    FileNotFound(String),
    /// Permission denied when accessing file
    PermissionDenied(String),
    /// File contains invalid UTF-8 encoding
    InvalidEncoding(String),
    /// Line number is out of bounds
    LineOutOfBounds(usize, usize), // (requested, total)
    /// File was modified by another process
    ConcurrentModification(String),
    /// Unable to write changes to file
    WriteFailure(String),
    /// Invalid command-line arguments
    InvalidArguments(String),
    /// I/O error occurred
    IoError(std::io::Error),
}

impl fmt::Display for GliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GliError::FileNotFound(path) => {
                write!(f, "Error: File not found: {}\n\nSuggestion: Create the file with:\n  touch .gitleaksignore", path)
            }
            GliError::PermissionDenied(path) => {
                write!(f, "Error: Permission denied: {}\n\nSuggestion: Check file permissions with:\n  ls -l {}", path, path)
            }
            GliError::InvalidEncoding(path) => {
                write!(f, "Error: File contains invalid UTF-8: {}", path)
            }
            GliError::LineOutOfBounds(requested, total) => {
                write!(
                    f,
                    "Error: Line {} is out of bounds (file has {} lines)\n\nValid range: 1-{}",
                    requested, total, total
                )
            }
            GliError::ConcurrentModification(path) => {
                write!(f, "Warning: File was modified by another process: {}", path)
            }
            GliError::WriteFailure(msg) => {
                write!(f, "Error: Unable to save changes: {}", msg)
            }
            GliError::InvalidArguments(msg) => {
                write!(f, "Error: Invalid arguments: {}", msg)
            }
            GliError::IoError(err) => {
                write!(f, "I/O Error: {}", err)
            }
        }
    }
}

impl std::error::Error for GliError {}

impl From<std::io::Error> for GliError {
    fn from(err: std::io::Error) -> Self {
        GliError::IoError(err)
    }
}

pub type Result<T> = std::result::Result<T, GliError>;
