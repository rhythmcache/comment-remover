//! Configuration management for the comment remover application.
//!
//! This module handles loading configuration from TOML files and merging it with
//! command-line arguments. It defines two primary structs:
//!
//! - [`Config`]: Raw configuration deserialized from a TOML file.
//! - [`ResolvedConfig`]: Final configuration after merging file and CLI values,
//!   ready for use by the processing engine.
//!
//! # Examples
//!
//! Loading a configuration file:
//!
//! ```
//! use comment_remover::config::Config;
//! use std::path::Path;
//!
//! let config = Config::from_file(Path::new("config.toml")).unwrap();
//! ```
//!
//! Merging with CLI arguments:
//!
//! ```
//! use comment_remover::config::{Config, ResolvedConfig};
//!
//! let file_config = Config::default();
//! let resolved = file_config.merge_with_cli(
//!     Some("rust".to_string()),  // cli_language
//!     None,                      // cli_collapse
//!     true,                      // cli_recursive
//!     None,                      // cli_output_dir
//!     Some(4),                    // cli_threads
//!     false,                      // cli_in_place
//!     false,                      // cli_dry_run
//!     true,                       // cli_diff
//!     false,                      // cli_json
//!     false,                      // cli_force
//! );
//! ```

use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{AppError, Result};

/// Raw configuration deserialized from a TOML file.
///
/// All fields are optional, allowing partial configuration files. Missing fields
/// are typically overridden by command-line arguments or fall back to defaults.
///
/// The struct is designed to be used with `serde` and includes `deny_unknown_fields`
/// to catch typos in configuration files.
#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    /// Programming language to process (e.g., "rust", "python", "javascript").
    /// If not specified, the language will be inferred from file extensions,
    /// or must be provided via CLI.
    pub language: Option<String>,

    /// Maximum number of consecutive newlines to retain after collapsing whitespace.
    /// When `Some(n)`, sequences of empty lines longer than `n` are reduced to `n`.
    /// If `None`, no whitespace collapsing is performed.
    pub collapse_whitespace: Option<usize>,

    /// Whether to process directories recursively.
    /// If `true`, when a directory is given as input, all files inside it are processed.
    /// This can be overridden by CLI flag.
    pub recursive: Option<bool>,

    /// Directory where output files should be written when not modifying in-place.
    /// If not specified, output defaults to stdout (for single files) or in-place editing.
    pub output_dir: Option<PathBuf>,

    /// Number of threads to use for parallel processing.
    /// If not specified, the number of CPU cores is used.
    pub threads: Option<usize>,
}

impl Config {
    /// Loads a configuration from a TOML file at the given path.
    ///
    /// # Arguments
    /// * `path` - Path to the TOML configuration file.
    ///
    /// # Returns
    /// * `Ok(Config)` if the file exists and contains valid TOML.
    /// * `Err(AppError::Io)` if the file cannot be read.
    /// * `Err(AppError::Config)` if the TOML is malformed or contains unknown fields.
    ///
    /// # Examples
    ///
    /// ```
    /// use comment_remover::config::Config;
    /// use std::path::Path;
    ///
    /// match Config::from_file(Path::new("config.toml")) {
    ///     Ok(cfg) => println!("Loaded config: {:?}", cfg),
    ///     Err(e) => eprintln!("Failed to load config: {}", e),
    /// }
    /// ```
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path).map_err(|e| AppError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;
        toml::from_str(&content)
            .map_err(|e| AppError::Config(format!("Failed to parse TOML: {}", e)))
    }

    /// Merges the file-based configuration with command-line arguments to produce a
    /// [`ResolvedConfig`] ready for use.
    ///
    /// Command-line arguments take precedence over values from the file. Boolean flags
    /// are combined with logical OR when applicable (e.g., `recursive` is true if either
    /// the file or CLI sets it true). The `dry_run` flag is also set true if `diff` is
    /// requested, as diffing implies no actual writes.
    ///
    /// # Arguments
    ///
    /// * `cli_language`   - Language override from CLI (`-l` / `--language`).
    /// * `cli_collapse`   - Whitespace collapse value from CLI (`-c` / `--collapse-whitespace`).
    /// * `cli_recursive`  - Recursive flag from CLI (`-r` / `--recursive`).
    /// * `cli_output_dir` - Output directory from CLI (`--output-dir`).
    /// * `cli_threads`    - Thread count from CLI (`--threads`).
    /// * `cli_in_place`   - In‑place editing flag from CLI (`--in-place`).
    /// * `cli_dry_run`    - Dry‑run flag from CLI (`--dry-run`).
    /// * `cli_diff`       - Diff flag from CLI (`--diff`).
    /// * `cli_json`       - JSON output flag from CLI (`--json`).
    /// * `cli_force`      - Force flag from CLI (`--force`).
    ///
    /// # Returns
    ///
    /// A [`ResolvedConfig`] containing the final, merged configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// use comment_remover::config::{Config, ResolvedConfig};
    ///
    /// let file_cfg = Config {
    ///     language: Some("python".to_string()),
    ///     collapse_whitespace: Some(2),
    ///     recursive: Some(false),
    ///     output_dir: None,
    ///     threads: None,
    /// };
    ///
    /// let resolved = file_cfg.merge_with_cli(
    ///     Some("rust".to_string()), // CLI overrides language
    ///     None,                     // no CLI collapse
    ///     true,                     // CLI sets recursive true
    ///     Some("out".into()),       // CLI output dir
    ///     Some(8),                   // CLI threads
    ///     false,
    ///     false,
    ///     false,
    ///     false,
    ///     false,
    /// );
    ///
    /// assert_eq!(resolved.language, Some("rust".to_string()));
    /// assert_eq!(resolved.recursive, true); // CLI overrides file false
    /// assert_eq!(resolved.collapse, Some(2)); // from file
    /// ```
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

/// Final, resolved configuration after merging file and CLI values.
///
/// This struct contains all settings necessary for the comment removal engine.
/// It is created by [`Config::merge_with_cli`] and used throughout the processing
/// pipeline to determine behaviour such as language detection, output handling,
/// parallelism, and logging.
#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    /// Programming language to process. If `None`, language will be inferred from
    /// file extensions.
    pub language: Option<String>,

    /// Maximum consecutive newlines to retain after collapsing whitespace.
    /// `None` means no collapsing is applied.
    pub collapse: Option<usize>,

    /// Whether to traverse directories recursively.
    pub recursive: bool,

    /// Directory for writing output files. If `None` and `in_place` is false,
    /// output goes to stdout (only valid for a single input file).
    pub output_dir: Option<PathBuf>,

    /// Number of threads to use for parallel processing. If `None`, the engine
    /// will use the number of CPU cores.
    pub threads: Option<usize>,

    /// Whether to modify files in-place (overwrite original).
    pub in_place: bool,

    /// If true, perform a dry run: no files are actually written.
    /// Automatically true if `diff` is true.
    pub dry_run: bool,

    /// If true, show a diff between original and processed content instead of
    /// writing output.
    pub diff: bool,

    /// If true, output results in JSON format instead of human-readable messages.
    pub json: bool,

    /// If true, continue processing even if some files fail, and suppress the
    /// "No files processed" error when all files fail.
    pub force: bool,
}
