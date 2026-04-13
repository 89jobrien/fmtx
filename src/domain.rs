use std::path::Path;
use thiserror::Error;

/// A single formatter command step.
#[derive(Debug, Clone)]
pub struct CommandStep {
    pub command: String,
    pub args: Vec<String>,
    /// Additional args appended when running in --check mode.
    pub check_args: Vec<String>,
}

/// One or more formatter steps registered for a file extension.
#[derive(Debug, Clone)]
pub struct FormatterEntry {
    pub steps: Vec<CommandStep>,
}

#[derive(Debug, Error)]
pub enum FormatterError {
    #[error("file has no extension: {0}")]
    NoExtension(String),

    #[error("config error: {0}")]
    Config(String),

    #[error("formatter '{command}' failed with exit code {code}")]
    FormatterFailed { command: String, code: i32 },

    #[error("formatter '{command}' was terminated by a signal")]
    FormatterSignaled { command: String },
}

pub trait Formatter {
    fn format(&self, path: &Path, check: bool) -> Result<(), FormatterError>;
}

pub fn extension_of(path: &Path) -> Result<String, FormatterError> {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .ok_or_else(|| FormatterError::NoExtension(path.display().to_string()))
}
