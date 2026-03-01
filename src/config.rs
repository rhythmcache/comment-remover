use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

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
        let content = fs::read_to_string(path).map_err(|e| AppError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;
        toml::from_str(&content)
            .map_err(|e| AppError::Config(format!("Failed to parse TOML: {}", e)))
    }

    pub fn merge_with_cli(
        &self,
        cli_language: Option<String>,
        cli_collapse: Option<usize>,
        cli_recursive: bool,
        cli_output_dir: Option<PathBuf>,
        cli_threads: Option<usize>,
        cli_in_place: bool,
        cli_dry_run: bool,
        cli_diff: bool,
        cli_json: bool,
        cli_force: bool,
    ) -> ResolvedConfig {
        ResolvedConfig {
            language: cli_language.or_else(|| self.language.clone()),
            collapse: cli_collapse.or(self.collapse_whitespace),
            recursive: cli_recursive || self.recursive.unwrap_or(false),
            output_dir: cli_output_dir.or_else(|| self.output_dir.clone()),
            threads: cli_threads.or(self.threads),
            in_place: cli_in_place,
            dry_run: cli_dry_run || cli_diff,
            diff: cli_diff,
            json: cli_json,
            force: cli_force,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    pub language: Option<String>,
    pub collapse: Option<usize>,
    pub recursive: bool,
    pub output_dir: Option<PathBuf>,
    pub threads: Option<usize>,
    pub in_place: bool,
    pub dry_run: bool,
    pub diff: bool,
    pub json: bool,
    pub force: bool,
}
