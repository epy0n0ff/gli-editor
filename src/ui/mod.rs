/// UI components for gli-editor (Ratatui-based TUI)
///
/// This module contains the terminal user interface implementation.
pub mod editor;
pub mod navigation;
pub mod styles;
pub mod viewer;

pub use editor::EditorWidget;
pub use navigation::NavigationWidget;
pub use styles::Styles;
pub use viewer::ViewerWidget;
