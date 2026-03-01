use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("Tree-sitter error: {0}")]
    TreeSitter(String),

    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),

    #[error("Query error: {0}")]
    Query(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("No files processed")]
    NoFiles,

    #[error("Directory not allowed without --recursive: {0}")]
    DirectoryNotAllowed(String),

    #[error("Language must be specified for stdin input (use -l/--language)")]
    StdinLanguageRequired,

    #[error("Cannot output multiple files to stdout without --in-place")]
    MultipleFilesStdout,
}

pub type Result<T> = std::result::Result<T, AppError>;
