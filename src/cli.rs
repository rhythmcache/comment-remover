//! Command-line interface for the comment remover.
//!
//! This module defines the CLI using [`clap`] and implements the main application
//! logic. It handles argument parsing, configuration loading, file collection,
//! parallel processing, and output reporting.
//!
//! The primary entry point is [`Cli::run`], which is called from `main.rs`.
//!
//! # Structure
//!
//! * [`Cli`] - The command-line arguments struct.
//! * [`ProcessorConfig`] - Internal configuration for processing a single file.
//! * Helper functions: [`setup_logging`], [`read_stdin`], [`process_results`],
//!   [`report_results`].
//!
//! # Examples
//!
//! The CLI is not intended to be used programmatically, but you can simulate
//! argument parsing:
//!
//! ```
//! use clap::Parser;
//! use comment_remover::cli::Cli;
//!
//! let cli = Cli::parse_from(["comment-remover", "file.rs", "--in-place"]);
//! ```

use clap::Parser;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use crate::config::{Config, ResolvedConfig};
use crate::core::language::TreeSitterLanguage;
use crate::core::remover::CommentRemover;
use crate::error::{AppError, Result, io_error};
use crate::io::{collect_files, create_parent_dir, read_file, write_file};
use serde_json::json;
use std::io::{self, Read};

/// Command-line arguments for the comment remover.
///
/// This struct is parsed using `clap` and contains all options that can be
/// passed to the program. Each field corresponds to a flag or argument.
#[derive(Parser, Debug)]
#[command(author, version, about = "Remove comments from source code files using tree-sitter", long_about = None)]
pub struct Cli {
    /// Input files or directories to process.
    ///
    /// If no files are given, the program reads from stdin (requires `--language`).
    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,

    /// Programming language of the input.
    ///
    /// If not specified, the language is inferred from the file extension.
    /// Required when reading from stdin.
    #[arg(short, long, value_name = "LANG")]
    pub language: Option<String>,

    /// Edit files in-place (overwrite original).
    ///
    /// If not set, output is written to stdout (only for a single file) or
    /// to `--output-dir`.
    #[arg(short, long)]
    pub in_place: bool,

    /// Collapse consecutive blank lines to at most N.
    ///
    /// After removing comments, sequences of empty lines longer than N are
    /// reduced to N newlines. If not specified, no collapsing is performed.
    #[arg(short, long, value_name = "N")]
    pub collapse_whitespace: Option<usize>,

    /// Process directories recursively.
    ///
    /// When a directory is given as input, all files inside it (and its
    /// subdirectories) are processed.
    #[arg(short, long)]
    pub recursive: bool,

    /// Write output files into this directory, preserving the relative path.
    ///
    /// For example, if input is `src/main.rs` and output dir is `out/`, the
    /// result is written to `out/src/main.rs`.
    #[arg(long, value_name = "DIR")]
    pub output_dir: Option<PathBuf>,

    /// Perform a dry run: do not write any files, only log what would be done.
    #[arg(long)]
    pub dry_run: bool,

    /// Show a diff between original and processed content instead of writing.
    ///
    /// Implies `--dry-run`.
    #[arg(long)]
    pub diff: bool,

    /// Number of threads to use for parallel processing.
    ///
    /// If not specified, uses the number of CPU cores.
    #[arg(long, value_name = "N")]
    pub threads: Option<usize>,

    /// Verbosity level (repeat for more detail, e.g., -v, -vv, -vvv).
    ///
    /// * -v: info
    /// * -vv: debug
    /// * -vvv: trace
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Suppress all output except errors.
    ///
    /// Overrides verbosity.
    #[arg(short, long)]
    pub quiet: bool,

    /// Output results in JSON format.
    ///
    /// For summary statistics and failures, JSON is printed instead of
    /// human-readable logs.
    #[arg(long)]
    pub json: bool,

    /// Path to a TOML configuration file.
    ///
    /// Values in the file are overridden by command-line arguments.
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Force processing: continue even if some files fail, and suppress the
    /// "No files processed" error when all files fail.
    #[arg(short, long)]
    pub force: bool,
}

impl Cli {
    /// Main entry point for the CLI after argument parsing.
    ///
    /// This method performs the following steps:
    /// 1. Sets up logging based on verbosity/quiet flags.
    /// 2. Loads the configuration file if provided.
    /// 3. Merges file configuration with CLI arguments into a [`ResolvedConfig`].
    /// 4. If no files are given, delegates to [`handle_stdin`](Self::handle_stdin).
    /// 5. Otherwise, collects input files (recursively if requested).
    /// 6. Validates output mode (e.g., multiple files cannot go to stdout).
    /// 7. Creates a Rayon thread pool with the requested number of threads.
    /// 8. Processes all files in parallel using [`process_one`](Self::process_one).
    /// 9. Aggregates results and reports them.
    ///
    /// # Returns
    ///
    /// * `Ok(())` on success or when no files were processed with `--force`.
    /// * `Err(AppError::NoFiles)` if all files failed and `--force` is not used.
    /// * Other errors as documented.
    pub fn run(self) -> Result<()> {
        setup_logging(self.verbose, self.quiet);

        let config = if let Some(path) = &self.config {
            let cfg = Config::from_file(path)?;
            info!("Loaded configuration from {}", path.display());
            Some(cfg)
        } else {
            None
        };

        // FIX: Clone language dan output_dir agar self tidak partially moved
        let resolved = if let Some(cfg) = config {
            cfg.merge_with_cli(
                self.language.clone(),
                self.collapse_whitespace,
                self.recursive,
                self.output_dir.clone(),
                self.threads,
                self.in_place,
                self.dry_run,
                self.diff,
                self.json,
                self.force,
            )
        } else {
            ResolvedConfig {
                language: self.language.clone(),
                collapse: self.collapse_whitespace,
                recursive: self.recursive,
                output_dir: self.output_dir.clone(),
                threads: self.threads,
                in_place: self.in_place,
                dry_run: self.dry_run || self.diff,
                diff: self.diff,
                json: self.json,
                force: self.force,
            }
        };

        if self.files.is_empty() {
            return self.handle_stdin(&resolved);
        }

        let language_override = if let Some(s) = &resolved.language {
            match TreeSitterLanguage::from_str(s) {
                Ok(lang) => {
                    debug!("Language override: {:?}", lang);
                    Some(lang)
                }
                Err(e) => {
                    error!("{}", e);
                    error!(
                        "Supported languages: {}",
                        TreeSitterLanguage::supported().join(", ")
                    );
                    return Err(AppError::UnsupportedLanguage(s.clone()));
                }
            }
        } else {
            None
        };

        let all_files = collect_files(&self.files, resolved.recursive)?;
        if all_files.is_empty() {
            warn!("No files found to process");
            return Ok(());
        }
        debug!("Collected {} files", all_files.len());

        let to_stdout = !resolved.in_place && resolved.output_dir.is_none() && all_files.len() == 1;

        if !resolved.in_place && resolved.output_dir.is_none() && all_files.len() > 1 {
            error!("Cannot output multiple files to stdout without --in-place or --output-dir");
            return Err(AppError::MultipleFilesStdout);
        }

        let num_threads = resolved.threads.unwrap_or_else(num_cpus::get);
        debug!("Using {} threads", num_threads);
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build()
            .map_err(|e| AppError::Config(format!("Failed to create thread pool: {}", e)))?;

        let processor_cfg = Arc::new(ProcessorConfig {
            language_override,
            collapse: resolved.collapse,
            in_place: resolved.in_place,
            output_dir: resolved.output_dir.clone(),
            dry_run: resolved.dry_run,
            diff: resolved.diff,
            to_stdout,
        });

        // Sekarang self masih utuh, bisa dipinjam di closure
        let results: Vec<Result<()>> = pool.install(|| {
            all_files
                .par_iter()
                .map(|path| self.process_one(path, &processor_cfg))
                .collect()
        });

        let (success, failed, errors) = process_results(results);
        report_results(success, failed, &errors, resolved.json);

        if failed > 0 && !resolved.force {
            Err(AppError::NoFiles)
        } else {
            Ok(())
        }
    }

    /// Handles the case where input is read from stdin.
    ///
    /// Requires that `resolved.language` is set, otherwise returns
    /// [`AppError::StdinLanguageRequired`]. Reads all data from stdin,
    /// processes it with a [`CommentRemover`], and prints the result
    /// (as plain text or JSON).
    fn handle_stdin(&self, resolved: &ResolvedConfig) -> Result<()> {
        let lang = match &resolved.language {
            Some(s) => TreeSitterLanguage::from_str(s).map_err(|e| {
                error!("{}", e);
                error!(
                    "Supported languages: {}",
                    TreeSitterLanguage::supported().join(", ")
                );
                AppError::UnsupportedLanguage(s.clone())
            })?,
            None => {
                error!("Language must be specified for stdin input (use -l/--language)");
                return Err(AppError::StdinLanguageRequired);
            }
        };

        let buffer = read_stdin().map_err(|e| io_error("<stdin>", e))?;
        debug!("Read {} bytes from stdin", buffer.len());

        let remover = CommentRemover::new(lang, resolved.collapse);
        let output = remover.process_str(&buffer)?;

        if resolved.json {
            let out = json!({ "result": output });
            println!("{}", serde_json::to_string_pretty(&out).unwrap());
        } else {
            print!("{}", output);
        }

        Ok(())
    }

    /// Processes a single file according to the given configuration.
    ///
    /// This method:
    /// 1. Determines the language (override or detection).
    /// 2. Reads the file content.
    /// 3. Removes comments using [`CommentRemover`].
    /// 4. If `diff` is true, shows a diff and returns.
    /// 5. If `dry_run` is true, logs and returns.
    /// 6. Otherwise, writes the output to the appropriate destination
    ///    (stdout, output dir, or in-place).
    fn process_one(&self, path: &Path, cfg: &ProcessorConfig) -> Result<()> {
        debug!("Processing {:?}", path);

        let language = if let Some(lang) = cfg.language_override {
            lang
        } else {
            TreeSitterLanguage::detect_from_path(path)
                .ok_or_else(|| AppError::UnsupportedLanguage(path.display().to_string()))?
        };

        let remover = CommentRemover::new(language, cfg.collapse);
        let input = read_file(path)?;
        let output = remover.process_str(&input)?;

        if cfg.diff {
            self.show_diff(path, &input, &output)?;
            return Ok(());
        }

        if cfg.dry_run {
            info!("[DRY RUN] Would process {}", path.display());
            return Ok(());
        }

        if cfg.to_stdout {
            print!("{}", output);
            Ok(())
        } else if let Some(out_dir) = &cfg.output_dir {
            let rel = path
                .strip_prefix(std::env::current_dir().map_err(|e| io_error(path, e))?)
                .unwrap_or(path);
            let out_path = out_dir.join(rel);
            create_parent_dir(&out_path)?;
            write_file(&out_path, &output)?;
            info!("Written to {}", out_path.display());
            Ok(())
        } else if cfg.in_place {
            write_file(path, &output)?;
            info!("Updated {}", path.display());
            Ok(())
        } else {
            Err(AppError::Config("No output destination specified".into()))
        }
    }

    /// Prints a unified diff between the original and modified content.
    ///
    /// Uses the `similar` crate to generate a line‑based diff. Each line is
    /// prefixed with `-`, `+`, or ` ` to indicate deletion, insertion, or
    /// unchanged.
    fn show_diff(&self, path: &Path, original: &str, modified: &str) -> Result<()> {
        use similar::{ChangeTag, TextDiff};
        let diff = TextDiff::from_lines(original, modified);
        println!("Diff for {}:", path.display());
        for change in diff.iter_all_changes() {
            let sign = match change.tag() {
                ChangeTag::Delete => "-",
                ChangeTag::Insert => "+",
                ChangeTag::Equal => " ",
            };
            print!("{}{}", sign, change);
        }
        println!();
        Ok(())
    }
}

/// Internal configuration passed to [`Cli::process_one`] for each file.
///
/// This struct is created once per run and shared across threads via an `Arc`.
/// It contains all settings that affect how a single file is processed.
struct ProcessorConfig {
    /// Override language (if specified via CLI or config).
    language_override: Option<TreeSitterLanguage>,
    /// Maximum consecutive newlines to keep (None = no collapse).
    collapse: Option<usize>,
    /// Whether to edit files in-place.
    in_place: bool,
    /// Output directory (if any).
    output_dir: Option<PathBuf>,
    /// If true, do not write any files.
    dry_run: bool,
    /// If true, show diff instead of writing.
    diff: bool,
    /// If true, output goes to stdout (only for single file).
    to_stdout: bool,
}

/// Reads all data from standard input into a string.
///
/// # Errors
///
/// Returns an `io::Error` if reading fails.
fn read_stdin() -> io::Result<String> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    Ok(buffer)
}

/// Aggregates a vector of `Result<()>` into counts of successes and failures,
/// and collects the errors.
///
/// # Returns
///
/// A tuple `(success_count, failure_count, errors)`.
fn process_results(results: Vec<Result<()>>) -> (usize, usize, Vec<AppError>) {
    let mut success = 0;
    let mut failed = 0;
    let mut errors = Vec::new();
    for res in results {
        match res {
            Ok(()) => success += 1,
            Err(e) => {
                failed += 1;
                errors.push(e);
            }
        }
    }
    (success, failed, errors)
}

/// Reports the final processing results to the user.
///
/// If `json` is true, prints a JSON object with success count, failure count,
/// and a list of error messages. Otherwise, uses `tracing` to log the summary
/// and any errors.
fn report_results(success: usize, failed: usize, errors: &[AppError], json: bool) {
    if json {
        let failures: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
        let summary = json!({
            "success": success,
            "failed": failed,
            "failures": failures,
        });
        let output = serde_json::to_string_pretty(&summary)
            .unwrap_or_else(|e| format!("{{\"error\": \"Failed to serialize JSON: {}\"}}", e));
        println!("{}", output);
    } else {
        if failed == 0 {
            info!("Successfully processed {} files", success);
        } else {
            error!("Processed: {}, Failed: {}", success, failed);
            for e in errors {
                error!("  - {}", e);
            }
        }
    }
}

use tracing_subscriber::{filter::EnvFilter, fmt};

/// Initializes the `tracing` subscriber based on verbosity and quiet flags.
///
/// Log level is determined as follows:
/// * If `quiet` is true → `error` (only errors are shown).
/// * Otherwise, based on `verbosity`:
///   * 0 → `warn`
///   * 1 → `info`
///   * 2 → `debug`
///   * ≥3 → `trace`
///
/// The level can also be overridden by the `RUST_LOG` environment variable.
pub fn setup_logging(verbosity: u8, quiet: bool) {
    let level = match (verbosity, quiet) {
        (_, true) => "error",
        (0, false) => "warn",
        (1, false) => "info",
        (2, false) => "debug",
        _ => "trace",
    };

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("comment_remover={}", level)));

    fmt::Subscriber::builder()
        .with_env_filter(filter)
        .with_writer(std::io::stdout)
        .init();
}