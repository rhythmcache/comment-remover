//! Configuration file support for the comment remover.
//!
//! This module provides functionality to load and merge settings from a TOML
//! configuration file. It allows users to persist common options, reducing
//! the need to specify them repeatedly on the command line.
//!
//! The configuration file is optional. If provided via the `--config <FILE>`
//! command‑line flag, its values are merged with the CLI arguments, with CLI
//! values taking precedence. This enables flexible setups where a project can
//! have a `.rmcm.toml` file containing defaults, and users can override them
//! per invocation.
//!
//! # Configuration File Format
//!
//! The file uses TOML syntax. All fields are optional. Unknown fields will
//! cause an error (to catch typos). The supported fields are:
//!
//! * `language` – Default language (string, e.g., `"rust"`, `"python"`).
//! * `collapse_whitespace` – Maximum number of consecutive blank lines to keep
//!   after comment removal (integer ≥ 0).
//! * `recursive` – Whether to process directories recursively (boolean).
//! * `output_dir` – Directory where processed files should be written (string, path).
//! * `threads` – Number of threads to use for parallel processing (integer).
//!
//! ## Example `.rmcm.toml`
//!
//! ```toml
//! language = "rust"
//! collapse_whitespace = 1
//! recursive = true
//! output_dir = "cleaned"
//! threads = 4
//! ```
//!
//! # Integration with CLI
//!
//! The typical flow is:
//!
//! 1. Parse command‑line arguments, obtaining optional values for each setting.
//! 2. If a config file was specified, load it with [`Config::from_file`].
//! 3. Merge the loaded config (if any) with the CLI values using
//!    [`Config::merge`], producing final settings.
//! 4. Use those final settings to drive the comment removal process.
//!
//! # Error Handling
//!
//! Loading a config file can fail due to I/O errors (file not found,
//! permission denied) or TOML parsing errors (invalid syntax, unknown fields,
//! incorrect types). These are returned as [`AppError::Io`] or
//! [`AppError::Config`] respectively.

use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use toml;

use crate::error::{AppError, Result};

/// All settings that can be specified in a configuration file.
///
/// Each field is optional because the config file may not contain every
/// option; unspecified fields will be left as `None` and the CLI values
/// (or defaults) will be used instead.
///
/// The struct is deserializable from TOML using `serde`. Field names are
/// in `snake_case` to match typical TOML conventions. The `deny_unknown_fields`
/// attribute ensures that any unknown field in the TOML causes an error,
/// preventing silent typos.
#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    /// Default language to use if not overridden by CLI or file extension.
    ///
    /// Should be a string like `"rust"`, `"python"`, etc. The value must
    /// match one of the supported languages (case‑insensitive) and the
    /// corresponding feature must be enabled. If the language is not
    /// available, an error will be raised during processing (not during
    /// config loading, because we only store the string here).
    pub language: Option<String>,

    /// Collapse consecutive blank lines to at most this number.
    ///
    /// If `Some(n)`, after removing comments, any run of blank lines
    /// longer than `n` will be reduced to exactly `n` blank lines. A value
    /// of `0` removes all blank lines. If `None`, no whitespace collapsing
    /// is performed (equivalent to `usize::MAX`).
    pub collapse_whitespace: Option<usize>,

    /// Process directories recursively.
    ///
    /// If `Some(true)`, when a directory is given as input, all files
    /// inside (and subdirectories) are processed. If `Some(false)`,
    /// directories are not allowed unless explicitly listed. If `None`,
    /// the CLI’s `--recursive` flag (or its absence) determines the
    /// behaviour.
    pub recursive: Option<bool>,

    /// Output directory for processed files (implies copy mode).
    ///
    /// If set, the program will write the cleaned output into this
    /// directory, preserving the relative path structure of input files.
    /// This is mutually exclusive with `--in-place` and writing to stdout.
    /// If `None`, output is handled according to other CLI flags.
    pub output_dir: Option<PathBuf>,

    /// Number of threads to use for parallel processing.
    ///
    /// If `Some(n)`, the program will use up to `n` threads to process
    /// multiple files concurrently. If `None`, a default (usually the
    /// number of CPU cores) is used.
    pub threads: Option<usize>,
}

impl Config {
    /// Loads a configuration from a TOML file at the given path.
    ///
    /// Reads the file, parses its contents as TOML, and constructs a
    /// `Config` struct. Unknown fields in the TOML will cause an error
    /// (thanks to `deny_unknown_fields`). All fields are optional.
    ///
    /// # Arguments
    ///
    /// * `path` – Path to the TOML configuration file.
    ///
    /// # Returns
    ///
    /// A `Config` instance populated with the values from the file.
    ///
    /// # Errors
    ///
    /// * [`AppError::Io`] if the file cannot be read (e.g., not found,
    ///   permission denied).
    /// * [`AppError::Config`] if the file contains invalid TOML or if it
    ///   includes fields that are not part of the `Config` struct.
    ///
    /// # Example
    ///
    /// ```
    /// # use comment_remover::config::Config;
    /// # use std::path::Path;
    /// match Config::from_file(Path::new(".rmcm.toml")) {
    ///     Ok(cfg) => println!("Loaded config: {:?}", cfg),
    ///     Err(e) => eprintln!("Failed to load config: {}", e),
    /// }
    /// ```
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path).map_err(AppError::Io)?;
        toml::from_str(&content)
            .map_err(|e| AppError::Config(format!("Failed to parse TOML: {}", e)))
    }

    /// Merges this configuration with CLI overrides.
    ///
    /// CLI values take precedence over config file values. This is typically
    /// used after loading a config file and before executing the command.
    /// For each setting:
    /// - If the CLI provided a value (i.e., `Some`), that value is used.
    /// - Otherwise, the value from the config (if any) is used.
    /// - If neither is provided, the result is `None` (meaning the program
    ///   should use its built‑in default or decide based on other rules).
    ///
    /// # Arguments
    ///
    /// * `cli_language` – Language provided via `-l`/`--language`.
    /// * `cli_collapse` – Value from `--collapse-whitespace`.
    /// * `cli_recursive` – Value from `--recursive`.
    /// * `cli_output_dir` – Value from `--output-dir`.
    /// * `cli_threads` – Value from `--threads`.
    ///
    /// # Returns
    ///
    /// A tuple of final values, where `Some` from CLI overrides `Some` from config.
    ///
    /// # Example
    ///
    /// ```
    /// # use comment_remover::config::Config;
    /// # use std::path::PathBuf;
    /// let cfg = Config {
    ///     language: Some("rust".to_string()),
    ///     collapse_whitespace: Some(2),
    ///     recursive: Some(false),
    ///     output_dir: Some(PathBuf::from("out")),
    ///     threads: Some(4),
    /// };
    ///
    /// let (lang, collapse, recursive, out_dir, threads) = cfg.merge(
    ///     Some("python".to_string()), // CLI overrides language
    ///     None,                        // no CLI collapse override
    ///     Some(true),                   // CLI recursive=true overrides false
    ///     None,
    ///     Some(8),                       // CLI threads=8 overrides 4
    /// );
    ///
    /// assert_eq!(lang, Some("python".to_string()));
    /// assert_eq!(collapse, Some(2));
    /// assert_eq!(recursive, Some(true));
    /// assert_eq!(out_dir, Some(PathBuf::from("out")));
    /// assert_eq!(threads, Some(8));
    /// ```
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
