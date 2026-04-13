use std::collections::HashMap;
use std::path::PathBuf;

use serde::Deserialize;

use crate::domain::{CommandStep, FormatterEntry, FormatterError};

// Raw single-command form: command = "...", args = [...], check_args = [...]
#[derive(Deserialize)]
struct RawSingle {
    command: String,
    #[serde(default)]
    args: Vec<String>,
    #[serde(default)]
    check_args: Vec<String>,
}

// Raw multi-command form: commands = [{ command = "...", ... }, ...]
#[derive(Deserialize)]
struct RawMulti {
    commands: Vec<RawSingle>,
}

// Untagged enum so serde tries RawMulti first (it has `commands`),
// then falls back to RawSingle (it has `command`).
#[derive(Deserialize)]
#[serde(untagged)]
enum RawEntry {
    Multi(RawMulti),
    Single(RawSingle),
}

#[derive(Deserialize)]
struct RawConfig {
    extensions: HashMap<String, RawEntry>,
}

pub struct Config {
    entries: HashMap<String, FormatterEntry>,
}

impl Config {
    pub fn load() -> Result<Self, FormatterError> {
        let path = config_path();
        let raw = std::fs::read_to_string(&path)
            .map_err(|e| FormatterError::Config(format!("cannot read {}: {e}", path.display())))?;
        Self::from_str(&raw)
    }

    /// Parse config from a TOML string. Exposed for testing.
    pub fn from_str(raw: &str) -> Result<Self, FormatterError> {
        let parsed: RawConfig =
            toml::from_str(raw).map_err(|e| FormatterError::Config(e.to_string()))?;

        let entries = parsed
            .extensions
            .into_iter()
            .map(|(ext, raw)| {
                let steps = match raw {
                    RawEntry::Single(s) => vec![raw_single_to_step(s)],
                    RawEntry::Multi(m) => m.commands.into_iter().map(raw_single_to_step).collect(),
                };
                (ext.to_lowercase(), FormatterEntry { steps })
            })
            .collect();

        Ok(Self { entries })
    }

    pub fn get(&self, ext: &str) -> Option<&FormatterEntry> {
        self.entries.get(ext)
    }
}

fn raw_single_to_step(s: RawSingle) -> CommandStep {
    CommandStep {
        command: s.command,
        args: s.args,
        check_args: s.check_args,
    }
}

fn config_path() -> PathBuf {
    dirs_next::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("fmtx")
        .join("config.toml")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_command_form_parses() {
        let toml = r#"
[extensions.rs]
command = "rustfmt"
args = []
check_args = ["--check"]
"#;
        let config = Config::from_str(toml).expect("parse failed");
        let entry = config.get("rs").expect("no rs entry");
        assert_eq!(entry.steps.len(), 1);
        assert_eq!(entry.steps[0].command, "rustfmt");
        assert_eq!(entry.steps[0].check_args, ["--check"]);
    }

    #[test]
    fn multi_command_form_parses() {
        let toml = r#"
[extensions.rs]
commands = [
  { command = "rustfmt", args = [], check_args = ["--check"] },
  { command = "clippy-driver", args = [], check_args = [] },
]
"#;
        let config = Config::from_str(toml).expect("parse failed");
        let entry = config.get("rs").expect("no rs entry");
        assert_eq!(entry.steps.len(), 2);
        assert_eq!(entry.steps[0].command, "rustfmt");
        assert_eq!(entry.steps[1].command, "clippy-driver");
    }

    #[test]
    fn extension_is_lowercased() {
        let toml = r#"
[extensions.RS]
command = "rustfmt"
"#;
        let config = Config::from_str(toml).expect("parse failed");
        assert!(config.get("rs").is_some());
        assert!(config.get("RS").is_none());
    }

    #[test]
    fn missing_extension_returns_none() {
        let toml = r#"
[extensions.py]
command = "black"
"#;
        let config = Config::from_str(toml).expect("parse failed");
        assert!(config.get("rs").is_none());
    }

    #[test]
    fn multi_command_with_no_args_defaults() {
        let toml = r#"
[extensions.go]
commands = [
  { command = "gofmt" },
]
"#;
        let config = Config::from_str(toml).expect("parse failed");
        let entry = config.get("go").expect("no go entry");
        assert_eq!(entry.steps.len(), 1);
        assert!(entry.steps[0].args.is_empty());
        assert!(entry.steps[0].check_args.is_empty());
    }
}
