/// gli-editor: Terminal editor for .gitleaksignore files
///
/// This is the main entry point for the application.
mod app;
mod core;
mod error;
mod models;
mod ui;

use app::App;
use clap::Parser;
use error::{GliError, Result};
use std::path::PathBuf;

/// Terminal editor for .gitleaksignore files
#[derive(Parser, Debug)]
#[command(name = "gli-editor")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Terminal editor for .gitleaksignore files", long_about = None)]
struct Cli {
    /// Path to .gitleaksignore file
    #[arg(short, long, default_value = "./.gitleaksignore")]
    file: PathBuf,

    /// Line specification (e.g., 42, 10-50, 42+5)
    #[arg(short, long)]
    lines: Option<String>,

    /// Number of context lines around target line
    #[arg(short = 'C', long, default_value = "3")]
    context: usize,

    /// Launch in read-only mode (disable editing)
    #[arg(short, long)]
    read_only: bool,
}

/// Line specification for viewing
#[derive(Debug, Clone)]
pub enum LineSpec {
    /// Display entire file
    All,
    /// Display a single line with context
    Single { line: usize, context: usize },
    /// Display a line range (inclusive)
    Range { start: usize, end: usize },
}

impl LineSpec {
    /// Parse line specification from string
    ///
    /// Formats:
    /// - "42" -> Single line 42 with default context
    /// - "10-50" -> Range from line 10 to 50
    /// - "42+5" -> Single line 42 with 5 lines of context
    pub fn parse(spec: &str, default_context: usize) -> Result<Self> {
        let spec = spec.trim();

        // Check for range (e.g., "10-50")
        if let Some(dash_pos) = spec.find('-') {
            let start_str = &spec[..dash_pos];
            let end_str = &spec[dash_pos + 1..];

            let start = start_str.parse::<usize>().map_err(|_| {
                GliError::InvalidArguments(format!("Invalid start line: {}", start_str))
            })?;

            let end = end_str.parse::<usize>().map_err(|_| {
                GliError::InvalidArguments(format!("Invalid end line: {}", end_str))
            })?;

            if start > end {
                return Err(GliError::InvalidArguments(format!(
                    "Start line {} cannot be greater than end line {}",
                    start, end
                )));
            }

            return Ok(LineSpec::Range { start, end });
        }

        // Check for single line with custom context (e.g., "42+5")
        if let Some(plus_pos) = spec.find('+') {
            let line_str = &spec[..plus_pos];
            let context_str = &spec[plus_pos + 1..];

            let line = line_str.parse::<usize>().map_err(|_| {
                GliError::InvalidArguments(format!("Invalid line number: {}", line_str))
            })?;

            let context = context_str.parse::<usize>().map_err(|_| {
                GliError::InvalidArguments(format!("Invalid context: {}", context_str))
            })?;

            return Ok(LineSpec::Single { line, context });
        }

        // Single line with default context
        let line = spec.parse::<usize>().map_err(|_| {
            GliError::InvalidArguments(format!("Invalid line specification: {}", spec))
        })?;

        Ok(LineSpec::Single {
            line,
            context: default_context,
        })
    }

    /// Calculate the actual line range to display
    pub fn calculate_range(&self, total_lines: usize) -> Result<(usize, usize)> {
        // Handle empty files
        if total_lines == 0 {
            return Ok((0, 0));
        }

        match self {
            LineSpec::All => Ok((1, total_lines)),
            LineSpec::Single { line, context } => {
                if *line == 0 || *line > total_lines {
                    return Err(GliError::LineOutOfBounds(*line, total_lines));
                }

                let start = line.saturating_sub(*context).max(1);
                let end = (*line + *context).min(total_lines);
                Ok((start, end))
            }
            LineSpec::Range { start, end } => {
                if *start == 0 || *start > total_lines {
                    return Err(GliError::LineOutOfBounds(*start, total_lines));
                }
                if *end > total_lines {
                    return Err(GliError::LineOutOfBounds(*end, total_lines));
                }
                Ok((*start, *end))
            }
        }
    }
}

fn main() -> Result<()> {
    // Parse command-line arguments
    let cli = Cli::parse();

    // Parse line specification
    let line_spec = if let Some(ref lines_str) = cli.lines {
        LineSpec::parse(lines_str, cli.context)?
    } else {
        LineSpec::All
    };

    // Create and run application with parsed arguments
    let mut app = App::new(cli.file, line_spec, cli.read_only)?;
    app.run()?;

    Ok(())
}
