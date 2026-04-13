use std::path::Path;
use std::process::Command;

use crate::domain::{CommandStep, Formatter, FormatterEntry, FormatterError};

pub struct CommandRunner<'a> {
    pub entry: &'a FormatterEntry,
}

impl Formatter for CommandRunner<'_> {
    fn format(&self, path: &Path, check: bool) -> Result<(), FormatterError> {
        for step in &self.entry.steps {
            run_step(step, path, check)?;
        }
        Ok(())
    }
}

fn run_step(step: &CommandStep, path: &Path, check: bool) -> Result<(), FormatterError> {
    let mut cmd = Command::new(&step.command);
    cmd.args(&step.args);

    if check {
        cmd.args(&step.check_args);
    } else {
        cmd.args(&step.format_args);
    }

    cmd.arg(path);

    let status = cmd
        .status()
        .map_err(|e| FormatterError::Config(format!("failed to spawn '{}': {e}", step.command)))?;

    if status.success() {
        return Ok(());
    }

    match status.code() {
        Some(code) => Err(FormatterError::FormatterFailed {
            command: step.command.clone(),
            code,
        }),
        None => Err(FormatterError::FormatterSignaled {
            command: step.command.clone(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{CommandStep, FormatterEntry};
    use std::path::Path;

    fn entry_with_steps(steps: Vec<CommandStep>) -> FormatterEntry {
        FormatterEntry { steps }
    }

    fn true_step() -> CommandStep {
        CommandStep {
            command: "true".to_string(),
            args: vec![],
            format_args: vec![],
            check_args: vec![],
        }
    }

    fn false_step() -> CommandStep {
        CommandStep {
            command: "false".to_string(),
            args: vec![],
            format_args: vec![],
            check_args: vec![],
        }
    }

    #[test]
    fn single_step_success() {
        let entry = entry_with_steps(vec![true_step()]);
        let runner = CommandRunner { entry: &entry };
        assert!(runner.format(Path::new("/dev/null"), false).is_ok());
    }

    #[test]
    fn single_step_failure() {
        let entry = entry_with_steps(vec![false_step()]);
        let runner = CommandRunner { entry: &entry };
        let err = runner.format(Path::new("/dev/null"), false).unwrap_err();
        assert!(matches!(err, FormatterError::FormatterFailed { .. }));
    }

    #[test]
    fn multi_step_all_succeed() {
        let entry = entry_with_steps(vec![true_step(), true_step()]);
        let runner = CommandRunner { entry: &entry };
        assert!(runner.format(Path::new("/dev/null"), false).is_ok());
    }

    #[test]
    fn multi_step_stops_on_first_failure() {
        // First step fails; second step would also fail, but we only care that
        // the error reported is from the first (false) step.
        let entry = entry_with_steps(vec![false_step(), true_step()]);
        let runner = CommandRunner { entry: &entry };
        let err = runner.format(Path::new("/dev/null"), false).unwrap_err();
        assert!(matches!(
            err,
            FormatterError::FormatterFailed { ref command, .. } if command == "false"
        ));
    }

    #[test]
    fn multi_step_second_fails() {
        let entry = entry_with_steps(vec![true_step(), false_step()]);
        let runner = CommandRunner { entry: &entry };
        let err = runner.format(Path::new("/dev/null"), false).unwrap_err();
        assert!(matches!(
            err,
            FormatterError::FormatterFailed { ref command, .. } if command == "false"
        ));
    }
}
