/// Data models for gli-editor
///
/// This module contains the core data structures used throughout the application.
pub mod edit;
pub mod line;
pub mod pattern;

pub use edit::EditOperation;
pub use line::{Line, LineRange};
pub use pattern::PatternType;
