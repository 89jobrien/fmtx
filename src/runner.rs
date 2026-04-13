use std::path::Path;
use std::process::Command;

use crate::domain::{Formatter, FormatterEntry, FormatterError};

pub struct CommandRunner<'a> {
    pub entry: &'a FormatterEntry,
}

impl Formatter for CommandRunner<'_> {
    fn format(&self, path: &Path, check: bool) -> Result<(), FormatterError> {
        let mut cmd = Command::new(&self.entry.command);
        cmd.args(&self.entry.args);

        if check {
            cmd.args(&self.entry.check_args);
        }

        cmd.arg(path);

        let status = cmd.status().map_err(|e| {
            FormatterError::Config(format!("failed to spawn '{}': {e}", self.entry.command))
        })?;

        if status.success() {
            return Ok(());
        }

        match status.code() {
            Some(code) => Err(FormatterError::FormatterFailed {
                command: self.entry.command.clone(),
                code,
            }),
            None => Err(FormatterError::FormatterSignaled {
                command: self.entry.command.clone(),
            }),
        }
    }
}
