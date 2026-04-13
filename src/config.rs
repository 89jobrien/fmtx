use std::collections::HashMap;
use std::path::PathBuf;

use serde::Deserialize;

use crate::domain::{FormatterEntry, FormatterError};

#[derive(Deserialize)]
struct RawConfig {
    extensions: HashMap<String, RawEntry>,
}

#[derive(Deserialize)]
struct RawEntry {
    command: String,
    #[serde(default)]
    args: Vec<String>,
    #[serde(default)]
    check_args: Vec<String>,
}

pub struct Config {
    entries: HashMap<String, FormatterEntry>,
}

impl Config {
    pub fn load() -> Result<Self, FormatterError> {
        let path = config_path();
        let raw = std::fs::read_to_string(&path).map_err(|e| {
            FormatterError::Config(format!("cannot read {}: {e}", path.display()))
        })?;
        let parsed: RawConfig =
            toml::from_str(&raw).map_err(|e| FormatterError::Config(e.to_string()))?;

        let entries = parsed
            .extensions
            .into_iter()
            .map(|(ext, raw)| {
                let entry = FormatterEntry {
                    command: raw.command,
                    args: raw.args,
                    check_args: raw.check_args,
                };
                (ext.to_lowercase(), entry)
            })
            .collect();

        Ok(Self { entries })
    }

    pub fn get(&self, ext: &str) -> Option<&FormatterEntry> {
        self.entries.get(ext)
    }
}

fn config_path() -> PathBuf {
    dirs_next::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("fmtx")
        .join("config.toml")
}
