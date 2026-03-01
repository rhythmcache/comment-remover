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

fn read_stdin() -> io::Result<String> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    Ok(buffer)
}

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

fn report_results(success: usize, failed: usize, errors: &[AppError], json: bool) {
    if json {
        let failures: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
        let summary = json!({
            "success": success,
            "failed": failed,
            "failures": failures,
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
            for e in errors {
                error!("  - {}", e);
            }
        }
    }
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