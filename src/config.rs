use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use toml;

use crate::error::{AppError, Result};

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub language: Option<String>,

    pub collapse_whitespace: Option<usize>,

    pub recursive: Option<bool>,

    pub output_dir: Option<PathBuf>,

    pub threads: Option<usize>,
}

impl Config {
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path).map_err(AppError::Io)?;
        toml::from_str(&content)
            .map_err(|e| AppError::Config(format!("Failed to parse TOML: {}", e)))
    }

    pub fn merge(
        &self,
        cli_language: Option<String>,
        cli_collapse: Option<usize>,
        cli_recursive: Option<bool>,
        cli_output_dir: Option<PathBuf>,
        cli_threads: Option<usize>,
    ) -> (
        Option<String>,
        Option<usize>,
        Option<bool>,
        Option<PathBuf>,
        Option<usize>,
    ) {
        (
            cli_language.or_else(|| self.language.clone()),
            cli_collapse.or(self.collapse_whitespace),
            cli_recursive.or(self.recursive),
            cli_output_dir.or_else(|| self.output_dir.clone()),
            cli_threads.or(self.threads),
        )
    }
}
