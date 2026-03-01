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

#[derive(Parser, Debug)]
#[command(author, version, about = "Remove comments from source code files using tree-sitter", long_about = None)]
pub struct Cli {
    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,

    #[arg(short, long, value_name = "LANG")]
    pub language: Option<String>,

    #[arg(short, long)]
    pub in_place: bool,

    #[arg(short, long, value_name = "N")]
    pub collapse_whitespace: Option<usize>,

    #[arg(short, long)]
    pub recursive: bool,

    #[arg(long, value_name = "DIR")]
    pub output_dir: Option<PathBuf>,

    #[arg(long)]
    pub dry_run: bool,

    #[arg(long)]
    pub diff: bool,

    #[arg(long, value_name = "N")]
    pub threads: Option<usize>,

    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    #[arg(short, long)]
    pub quiet: bool,

    #[arg(long)]
    pub json: bool,

    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    #[arg(short, long)]
    pub force: bool,
}

impl Cli {
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

struct ProcessorConfig {
    language_override: Option<TreeSitterLanguage>,

    collapse: Option<usize>,

    in_place: bool,

    output_dir: Option<PathBuf>,

    dry_run: bool,

    diff: bool,

    to_stdout: bool,
}

use tracing_subscriber::{filter::EnvFilter, fmt};

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
