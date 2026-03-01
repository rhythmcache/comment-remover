//! Command-line interface for the comment remover.
//!
//! This module defines the command-line argument parser using `clap` and
//! implements the main application logic in [`Cli::run`]. It handles the
//! entire workflow:
//!
//! - Parsing arguments provided by the user.
//! - Loading an optional TOML configuration file.
//! - Setting up logging based on verbosity flags.
//! - Collecting input files (with optional recursive directory traversal).
//! - Detecting or overriding the language for each file.
//! - Processing files either sequentially or in parallel (using Rayon).
//! - Writing output to stdout, in‑place, or to a specified output directory.
//! - Supporting dry‑run and diff modes.
//! - Producing JSON output for integration with other tools.
//!
//! # Usage
//!
//! The command-line interface is built with `clap` and supports a wide range
//! of options. Run `comment-remover --help` to see the full list.
//!
//! # Example
//!
//! Process a single Rust file and print the result to stdout:
//! ```bash
//! comment-remover main.rs
//! ```
//!
//! Process all `.py` files in a directory recursively, overwriting the originals:
//! ```bash
//! comment-remover --recursive --in-place src/
//! ```
//!
//! Use a configuration file and override the language:
//! ```bash
//! comment-remover --config .rmcm.toml --language rust file.c
//! ```
//!
//! # Integration
//!
//! The JSON output (`--json`) is designed for consumption by editors,
//! build tools, or scripts. When processing files, it prints a summary of
//! successes and failures. When reading from stdin, it wraps the cleaned
//! content in a JSON object.

use clap::Parser;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use crate::config::Config;
use crate::core::language::TreeSitterLanguage;
use crate::core::remover::CommentRemover;
use crate::error::{AppError, Result};
use crate::io::{collect_files, create_parent_dir, read_file, write_file};
use serde_json::json;
use std::io::{self, Read};

/// Command-line arguments for the comment remover.
///
/// This struct is parsed directly from `std::env::args()` using `clap`.
/// All fields correspond to command-line flags or positional arguments.
/// The `#[command]` attribute provides metadata for the help screen.
///
/// # Example
///
/// ```
/// use comment_remover::cli::Cli;
/// use clap::Parser;
///
/// let args = Cli::parse_from(["comment-remover", "--recursive", "src"]);
/// assert!(args.recursive);
/// assert_eq!(args.files, vec!["src".into()]);
/// ```
#[derive(Parser, Debug)]
#[command(author, version, about = "Remove comments from source code files using tree-sitter", long_about = None)]
pub struct Cli {
    /// Input files or directories to process.
    ///
    /// If no files are given, the program reads from standard input. In that
    /// case, the `--language` flag must be provided.
    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,

    /// Language override (auto-detected from extension if omitted).
    ///
    /// Use this when the language cannot be detected (e.g., stdin) or to
    /// override detection. Supported languages depend on build features.
    /// Run `comment-remover --help` to see the list of enabled languages.
    #[arg(short, long, value_name = "LANG")]
    pub language: Option<String>,

    /// Edit files in-place (overwrite original).
    ///
    /// Cannot be used with `--output-dir` or when processing multiple files
    /// without `--in-place` (since output would be interleaved on stdout).
    #[arg(short, long)]
    pub in_place: bool,

    /// Collapse consecutive blank lines to at most N.
    ///
    /// After removing comments, reduce runs of blank lines to at most N.
    /// Use 0 to remove all blank lines. If omitted, no collapsing is done.
    #[arg(short, long, value_name = "N")]
    pub collapse_whitespace: Option<usize>,

    /// Process directories recursively.
    ///
    /// When a directory is given as input, all files inside (and subdirectories)
    /// are processed. Without this flag, directories cause an error.
    #[arg(short, long)]
    pub recursive: bool,

    /// Output directory for processed files.
    ///
    /// If set, processed files are written to this directory, preserving the
    /// relative path structure of inputs. This implies `--in-place` is ignored.
    /// Cannot be used when processing stdin.
    #[arg(long, value_name = "DIR")]
    pub output_dir: Option<PathBuf>,

    /// Dry run: only print what would be done, without writing.
    ///
    /// Useful for checking which files would be affected.
    #[arg(long)]
    pub dry_run: bool,

    /// Show diff instead of writing (implies --dry-run).
    ///
    /// For each file, print a unified diff between the original and the
    /// cleaned version. Does not modify files.
    #[arg(long)]
    pub diff: bool,

    /// Number of threads to use for parallel processing.
    ///
    /// Defaults to the number of logical CPU cores.
    #[arg(long, value_name = "N")]
    pub threads: Option<usize>,

    /// Verbose output (use -vv for debug).
    ///
    /// - `-v`: info level
    /// - `-vv`: debug level
    /// - `-vvv`: trace level
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Quiet mode: suppress all output except errors.
    ///
    /// Overrides verbose.
    #[arg(short, long)]
    pub quiet: bool,

    /// Output results as JSON (for integration with other tools).
    ///
    /// When processing files, a JSON summary of successes and failures is printed.
    /// When processing stdin, the cleaned content is wrapped in a JSON object.
    #[arg(long)]
    pub json: bool,

    /// Configuration file (TOML format).
    ///
    /// Settings from the file are merged with command-line arguments;
    /// command-line arguments take precedence.
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Force continue even if some files fail.
    ///
    /// By default, the program exits on the first error. With `--force`,
    /// it processes as many files as possible and reports failures at the end.
    #[arg(short, long)]
    pub force: bool,
}

impl Cli {
    /// Main entry point for the CLI.
    ///
    /// This method orchestrates the entire process:
    ///
    /// 1. **Logging setup** – Initialises `tracing` based on `verbose`/`quiet`.
    /// 2. **Configuration loading** – If `--config` is provided, loads and
    ///    parses the TOML file.
    /// 3. **Merging** – Combines CLI arguments with config values (CLI wins).
    /// 4. **Stdin handling** – If no files are given, reads from stdin.
    /// 5. **File collection** – Expands input paths into a flat list of
    ///    regular files, respecting `--recursive`.
    /// 6. **Thread pool setup** – Configures Rayon with the requested thread
    ///    count.
    /// 7. **Parallel processing** – Processes each file in parallel using the
    ///    shared configuration.
    /// 8. **Reporting** – Prints a summary of successes and failures,
    ///    optionally in JSON.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The configuration file cannot be read or parsed.
    /// - The specified language is not supported.
    /// - No files are found and stdin is not being used.
    /// - A directory is given without `--recursive`.
    /// - Multiple files would be written to stdout without `--in-place`.
    /// - Any file processing fails and `--force` is not used (the first error
    ///   is returned).
    ///
    /// # Example
    ///
    /// This method is typically called from `main`:
    ///
    /// ```no_run
    /// use comment_remover::cli::Cli;
    /// use clap::Parser;
    ///
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let cli = Cli::parse();
    ///     cli.run()?;
    ///     Ok(())
    /// }
    /// ```
    pub fn run(self) -> Result<()> {
        setup_logging(self.verbose, self.quiet);

        let config = if let Some(path) = &self.config {
            let cfg = Config::from_file(path)?;
            info!("Loaded configuration from {}", path.display());
            Some(cfg)
        } else {
            None
        };

        let (
            lang_str,
            collapse,
            recursive,
            output_dir,
            threads,
            in_place,
            dry_run,
            diff,
            json,
            force,
        ) = self.merge_with_config(config.as_ref());

        if self.files.is_empty() {
            return self.handle_stdin(lang_str, collapse, json);
        }

        let language_override = if let Some(s) = lang_str {
            match TreeSitterLanguage::from_str(&s) {
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
                    return Err(AppError::UnsupportedLanguage(s));
                }
            }
        } else {
            None
        };

        let all_files = collect_files(&self.files, recursive)?;
        if all_files.is_empty() {
            warn!("No files found to process");
            return Ok(());
        }
        debug!("Collected {} files", all_files.len());

        let to_stdout = !in_place && output_dir.is_none() && all_files.len() == 1;

        if !in_place && output_dir.is_none() && all_files.len() > 1 {
            error!("Cannot output multiple files to stdout without --in-place or --output-dir");
            return Err(AppError::MultipleFilesStdout);
        }

        let num_threads = threads.unwrap_or_else(num_cpus::get);
        debug!("Using {} threads", num_threads);
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build()
            .map_err(|e| AppError::Config(format!("Failed to create thread pool: {}", e)))?;

        let processor_cfg = Arc::new(ProcessorConfig {
            language_override,
            collapse,
            in_place,
            output_dir: output_dir.clone(),
            dry_run,
            diff,
            to_stdout,
        });

        let results = pool.install(|| {
            all_files
                .par_iter()
                .map(|path| self.process_one(path, &processor_cfg))
                .collect::<Vec<_>>()
        });

        let mut success = 0;
        let mut failed = 0;
        let mut failures = Vec::new();

        for res in results {
            match res {
                Ok(()) => success += 1,
                Err(e) => {
                    failed += 1;
                    failures.push(e);
                }
            }
        }

        /// Reports the results of processing, either as plain text or JSON.
        ///
        /// # Arguments
        ///
        /// * `success` – Number of successfully processed files.
        /// * `failed` – Number of files that failed.
        /// * `failures` – A slice of error-like items to display.
        /// * `json` – Whether to output JSON.
        fn report_results(success: usize, failed: usize, failures: &[impl ToString], json: bool) {
            if json {
                let summary = json!({
                    "success": success,
                    "failed": failed,
                    "failures": failures.iter().map(ToString::to_string).collect::<Vec<_>>(),
                });

                let output = serde_json::to_string_pretty(&summary).unwrap_or_else(|e| {
                    format!("{{\"error\": \"Failed to serialize JSON: {}\"}}", e)
                });

                println!("{}", output);
            } else {
                if failed == 0 {
                    info!("Successfully processed {} files", success);
                } else {
                    error!("Processed: {}, Failed: {}", success, failed);
                    for e in failures {
                        error!("  - {}", e.to_string());
                    }
                }
            }
        }

        report_results(success, failed, &failures, json);

        if failed > 0 && !force {
            Err(AppError::NoFiles)
        } else {
            Ok(())
        }
    }

    /// Merge CLI arguments with configuration file values.
    ///
    /// CLI values take precedence over config values. Returns a tuple of all
    /// settings needed for the rest of the program.
    ///
    /// # Arguments
    ///
    /// * `config` – An optional reference to a loaded `Config`. If `None`,
    ///   default values are used.
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// `(language, collapse, recursive, output_dir, threads, in_place, dry_run, diff, json, force)`.
    fn merge_with_config(
        &self,
        config: Option<&Config>,
    ) -> (
        Option<String>,
        Option<usize>,
        bool,
        Option<PathBuf>,
        Option<usize>,
        bool,
        bool,
        bool,
        bool,
        bool,
    ) {
        let default_config = Config::default();
        let cfg = config.unwrap_or(&default_config);

        let lang = self.language.clone().or_else(|| cfg.language.clone());
        let collapse = self.collapse_whitespace.or(cfg.collapse_whitespace);
        let recursive = self.recursive || cfg.recursive.unwrap_or(false);
        let output_dir = self.output_dir.clone().or_else(|| cfg.output_dir.clone());
        let threads = self.threads.or(cfg.threads);

        (
            lang,
            collapse,
            recursive,
            output_dir,
            threads,
            self.in_place,
            self.dry_run || self.diff,
            self.diff,
            self.json,
            self.force,
        )
    }

    /// Handle the case where no files are given (read from stdin).
    ///
    /// This method reads all input from stdin, processes it with the specified
    /// language, and prints the result (optionally as JSON).
    ///
    /// # Arguments
    ///
    /// * `lang_str` – The language string from CLI/config (must be `Some`).
    /// * `collapse` – Optional blank‑line collapse limit.
    /// * `json` – Whether to wrap output in JSON.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::StdinLanguageRequired`] if `lang_str` is `None`.
    /// Returns [`AppError::UnsupportedLanguage`] if the language string is invalid.
    /// Returns I/O errors if reading from stdin fails.
    /// Returns parsing errors from [`CommentRemover::process_str`].
    fn handle_stdin(
        &self,
        lang_str: Option<String>,
        collapse: Option<usize>,
        json: bool,
    ) -> Result<()> {
        let lang = match lang_str {
            Some(s) => TreeSitterLanguage::from_str(&s).map_err(|e| {
                error!("{}", e);
                error!(
                    "Supported languages: {}",
                    TreeSitterLanguage::supported().join(", ")
                );
                AppError::UnsupportedLanguage(s)
            })?,
            None => {
                error!("Language must be specified for stdin input (use -l/--language)");
                return Err(AppError::StdinLanguageRequired);
            }
        };

        /// Reads all input from stdin into a string.
        fn read_stdin() -> io::Result<String> {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            Ok(buffer)
        }
        let buffer = read_stdin().map_err(AppError::Io)?;
        debug!("Read {} bytes from stdin", buffer.len());

        let remover = CommentRemover::new(lang, collapse);
        let output = remover.process_str(&buffer)?;

        if json {
            let out = serde_json::json!({ "result": output });
            println!("{}", serde_json::to_string_pretty(&out).unwrap());
        } else {
            print!("{}", output);
        }

        Ok(())
    }

    /// Process a single file according to the configuration.
    ///
    /// This function:
    /// - Determines the language (using override or file extension).
    /// - Reads the file.
    /// - Removes comments (and optionally collapses whitespace).
    /// - Depending on the mode, either shows a diff, does a dry‑run,
    ///   writes to stdout, overwrites the file, or writes to the output
    ///   directory.
    ///
    /// # Arguments
    ///
    /// * `path` – Path to the file to process.
    /// * `cfg` – Shared configuration for the processor.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The language cannot be detected (and no override was provided).
    /// - The file cannot be read or written.
    /// - Comment removal fails (parse error, query error).
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
                .strip_prefix(std::env::current_dir().map_err(AppError::Io)?)
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

    /// Show a unified diff between original and cleaned content.
    ///
    /// The diff is printed to stdout using the `similar` crate.
    ///
    /// # Arguments
    ///
    /// * `path` – Path of the file (used only for display).
    /// * `original` – Original file content.
    /// * `modified` – Content after comment removal.
    ///
    /// # Errors
    ///
    /// This function is infallible, but returns `Result` for consistency.
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

/// Configuration shared across threads during parallel processing.
///
/// This struct holds all settings that are constant for a given run and
/// are needed by [`Cli::process_one`]. It is wrapped in an `Arc` to be
/// shared safely across Rayon threads.
struct ProcessorConfig {
    /// Language override, if any.
    language_override: Option<TreeSitterLanguage>,

    /// Maximum number of consecutive blank lines to keep (if any).
    collapse: Option<usize>,

    /// Whether to overwrite original files.
    in_place: bool,

    /// Output directory (if any).
    output_dir: Option<PathBuf>,

    /// Dry run (no writes).
    dry_run: bool,

    /// Diff mode (implies dry run).
    diff: bool,
    /// Whether to write output to stdout (only for single file).
    to_stdout: bool,
}

use tracing_subscriber::{filter::EnvFilter, fmt};

/// Set up logging based on verbosity and quiet flags.
///
/// This function initialises the `tracing` subscriber with a filter that
/// reflects the desired log level. The level is determined as:
///
/// - If `quiet` is `true`, level is `error`.
/// - Otherwise, level depends on `verbosity`:
///   - 0: warn
///   - 1: info
///   - 2: debug
///   - ≥3: trace
///
/// The filter can also be overridden by the `RUST_LOG` environment variable.
///
/// # Arguments
///
/// * `verbosity` – The count of `-v` flags (0 to many).
/// * `quiet` – Whether `--quiet` was used.
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
