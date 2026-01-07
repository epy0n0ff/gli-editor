/// Core business logic for gli-editor
///
/// This module contains the file operations, parsing, and editing logic.
pub mod backup;
pub mod editor;
pub mod file_reader;
pub mod line_parser;

pub use backup::BackupManager;
pub use editor::EditorCore;
pub use file_reader::FileReader;
pub use line_parser::LineParser;
