//! Error types and result aliases for the comment remover application.
//!
//! This module defines the application-specific error enum [`AppError`] and a convenience
//! [`Result`] type alias. It also provides a helper function [`io_error`] to construct
//! I/O errors with associated paths.
//!
//! # Examples
//!
//! Basic usage of `Result` and `AppError`:
//!
//! ```
//! use comment_remover::error::{AppError, Result};
//!
//! fn process_file() -> Result<()> {
//!     // Simulate an error condition
//!     Err(AppError::NoFiles)
//! }
//! ```
//!
//! Using `io_error` to wrap an I/O error:
//!
//! ```
//! use comment_remover::error::io_error;
//! use std::fs::File;
//! use std::path::Path;
//!
//! fn read_file(path: &Path) -> Result<String, AppError> {
//!     std::fs::read_to_string(path).map_err(|e| io_error(path, e))
//! }
//! ```

use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// All possible errors that can occur in the comment remover application.
///
/// Errors are categorized by their origin: I/O operations, Tree-sitter parsing,
/// unsupported languages, query construction, configuration, and various
/// usage-related issues.
///
/// Each variant includes relevant context to help diagnose the problem.
#[derive(Error, Debug)]
pub enum AppError {
    /// An I/O error occurred while accessing a specific path.
    ///
    /// This variant includes the path that caused the error and the underlying
    /// [`std::io::Error`]. It is commonly returned by file reading/writing
    /// functions.
    ///
    /// # Example
    ///
    /// ```
    /// use comment_remover::error::{AppError, io_error};
    /// use std::fs::File;
    ///
    /// fn open_file(path: &str) -> Result<(), AppError> {
    ///     File::open(path).map_err(|e| io_error(path, e))?;
    ///     Ok(())
    /// }
    /// ```
    #[error("I/O error on {path}: {source}")]
    Io {
        /// The file or directory path where the I/O error occurred.
        path: PathBuf,
        /// The underlying I/O error.
        source: io::Error,
    },

    /// An error originating from the Tree-sitter library.
    ///
    /// This can happen when loading a grammar, setting a language for a parser,
    /// or during parsing itself (though parsing failures are covered by
    /// [`AppError::Parse`]).
    ///
    /// # Example
    ///
    /// ```
    /// use comment_remover::error::AppError;
    ///
    /// fn load_grammar() -> Result<(), AppError> {
    ///     // Simulate a Tree-sitter error
    ///     Err(AppError::TreeSitter("Failed to load language".to_string()))
    /// }
    /// ```
    #[error("Tree-sitter error: {0}")]
    TreeSitter(String),

    /// The specified programming language is not supported or not enabled in this build.
    ///
    /// Languages can be unsupported either because they are not included in the
    /// feature set of the build (e.g., compiled without the "python" feature) or
    /// because the language name/extension is unknown.
    ///
    /// # Example
    ///
    /// ```
    /// use comment_remover::error::AppError;
    ///
    /// fn detect_language(ext: &str) -> Result<(), AppError> {
    ///     match ext {
    ///         "rs" => Ok(()),
    ///         _ => Err(AppError::UnsupportedLanguage(ext.to_string())),
    ///     }
    /// }
    /// ```
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),

    /// An error occurred while creating or executing a Tree-sitter query.
    ///
    /// This usually indicates a malformed query string or an incompatibility
    /// between the query and the grammar.
    ///
    /// # Example
    ///
    /// ```
    /// use comment_remover::error::AppError;
    ///
    /// fn create_query() -> Result<(), AppError> {
    ///     // Invalid query syntax
    ///     Err(AppError::Query("Invalid query: unmatched parenthesis".to_string()))
    /// }
    /// ```
    #[error("Query error: {0}")]
    Query(String),

    /// A parsing error occurred while processing source code.
    ///
    /// This variant is returned when the Tree-sitter parser fails to produce a
    /// syntax tree for the given input. This can happen if the input contains
    /// syntax errors or if the parser encounters an unexpected situation.
    ///
    /// # Example
    ///
    /// ```
    /// use comment_remover::error::AppError;
    ///
    /// fn parse_code(code: &str) -> Result<(), AppError> {
    ///     if code.is_empty() {
    ///         return Err(AppError::Parse("Empty input".to_string()));
    ///     }
    ///     Ok(())
    /// }
    /// ```
    #[error("Parse error: {0}")]
    Parse(String),

    /// An error related to application configuration.
    ///
    /// This can be a malformed configuration file, missing required fields,
    /// or an invalid combination of command-line arguments.
    ///
    /// # Example
    ///
    /// ```
    /// use comment_remover::error::AppError;
    ///
    /// fn load_config(data: &str) -> Result<(), AppError> {
    ///     if !data.contains("language") {
    ///         return Err(AppError::Config("Missing 'language' field".to_string()));
    ///     }
    ///     Ok(())
    /// }
    /// ```
    #[error("Configuration error: {0}")]
    Config(String),

    /// No files were successfully processed.
    ///
    /// This error is returned when all processing attempts fail and the
    /// `--force` flag is not used. It indicates that the operation did not
    /// produce any successful output.
    ///
    /// # Example
    ///
    /// ```
    /// use comment_remover::error::AppError;
    ///
    /// fn process_all(files: &[&str]) -> Result<(), AppError> {
    ///     if files.is_empty() {
    ///         return Err(AppError::NoFiles);
    ///     }
    ///     Ok(())
    /// }
    /// ```
    #[error("No files processed")]
    NoFiles,

    /// A directory was provided without the `--recursive` flag.
    ///
    /// When a directory is given as input, the user must explicitly enable
    /// recursive traversal by passing `-r` or `--recursive`. This error
    /// prevents accidental processing of entire directory trees.
    ///
    /// # Example
    ///
    /// ```
    /// use comment_remover::error::AppError;
    ///
    /// fn handle_input(path: &std::path::Path, recursive: bool) -> Result<(), AppError> {
    ///     if path.is_dir() && !recursive {
    ///         return Err(AppError::DirectoryNotAllowed(path.display().to_string()));
    ///     }
    ///     Ok(())
    /// }
    /// ```
    #[error("Directory not allowed without --recursive: {0}")]
    DirectoryNotAllowed(String),

    /// The `--language` flag must be specified when reading from stdin.
    ///
    /// Because stdin provides no filename extension, the language cannot be
    /// auto-detected and must be provided manually using `-l` or `--language`.
    ///
    /// # Example
    ///
    /// ```
    /// use comment_remover::error::AppError;
    ///
    /// fn process_stdin(language: Option<&str>) -> Result<(), AppError> {
    ///     if language.is_none() {
    ///         return Err(AppError::StdinLanguageRequired);
    ///     }
    ///     Ok(())
    /// }
    /// ```
    #[error("Language must be specified for stdin input (use -l/--language)")]
    StdinLanguageRequired,

    /// Attempted to write multiple files to stdout, which is ambiguous.
    ///
    /// When more than one input file is given, the output must be directed
    /// either with `--in-place` or `--output-dir`; writing all files
    /// concatenated to stdout is not supported because it would mix outputs
    /// without separation.
    ///
    /// # Example
    ///
    /// ```
    /// use comment_remover::error::AppError;
    ///
    /// fn check_output_mode(file_count: usize, in_place: bool, out_dir: Option<&str>) -> Result<(), AppError> {
    ///     if file_count > 1 && !in_place && out_dir.is_none() {
    ///         return Err(AppError::MultipleFilesStdout);
    ///     }
    ///     Ok(())
    /// }
    /// ```
    #[error("Cannot output multiple files to stdout without --in-place")]
    MultipleFilesStdout,
}

/// A specialized `Result` type for comment remover operations.
///
/// This alias is used throughout the application to avoid repeating
/// `std::result::Result<T, AppError>`.
///
/// # Example
///
/// ```
/// use comment_remover::error::{Result, AppError};
///
/// fn safe_divide(a: f64, b: f64) -> Result<f64> {
///     if b == 0.0 {
///         Err(AppError::Config("Division by zero".to_string()))
///     } else {
///         Ok(a / b)
///     }
/// }
/// ```
pub type Result<T> = std::result::Result<T, AppError>;

/// Constructs an [`AppError::Io`] from a path and an I/O error.
///
/// This helper simplifies creating I/O errors that carry the path where the
/// error occurred. It is particularly useful in `map_err` chains when handling
/// I/O operations.
///
/// # Arguments
///
/// * `path` - The path that caused the I/O error. Can be any type that
///            implements `Into<PathBuf>` (e.g., `&str`, `&Path`, `PathBuf`).
/// * `err`  - The underlying [`std::io::Error`].
///
/// # Returns
///
/// An [`AppError::Io`] containing the given path and error.
///
/// # Example
///
/// ```
/// use comment_remover::error::io_error;
/// use std::fs::File;
/// use std::path::Path;
///
/// fn read_file_safe(path: &Path) -> Result<String, AppError> {
///     std::fs::read_to_string(path).map_err(|e| io_error(path, e))
/// }
/// ```
pub fn io_error(path: impl Into<PathBuf>, err: io::Error) -> AppError {
    AppError::Io {
        path: path.into(),
        source: err,
    }
}
