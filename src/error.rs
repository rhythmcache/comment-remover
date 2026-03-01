use std::io;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("I/O error on {path}: {source}")]
    Io { path: PathBuf, source: io::Error },

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

pub fn io_error(path: impl Into<PathBuf>, err: io::Error) -> AppError {
    AppError::Io {
        path: path.into(),
        source: err,
    }
}
