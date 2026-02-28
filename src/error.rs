//! Error types for the comment remover application.
//!
//! This module defines the central error type [`AppError`] used throughout the
//! application. All operations that can fail—such as file I/O, parsing with
//! tree‑sitter, configuration loading, and command‑line argument validation—
//! return a `Result<T>` alias that uses this error type.
//!
//! The error type is built with [`thiserror`], which automatically derives
//! the standard [`Error`] trait and provides convenient conversions from
//! lower‑level error types like [`std::io::Error`].
//!
//! # Organization
//!
//! - [`AppError`]: The main error enum, covering all failure modes.
//! - [`Result`]: A convenience type alias for `std::result::Result<T, AppError>`.
//!
//! # Example
//!
//! ```
//! use comment_remover::error::{AppError, Result};
//!
//! fn process_file(path: &str) -> Result<()> {
//!     if !path.ends_with(".rs") {
//!         return Err(AppError::UnsupportedLanguage(path.to_string()));
//!     }
//!     // ... actual processing ...
//!     Ok(())
//! }
//! ```

use std::io;
use thiserror::Error;

/// Central error type for the comment remover application.
///
/// This enum aggregates all possible errors that can occur during the
/// execution of the program. Each variant is annotated with a user‑facing
/// error message (via `#[error("...")]`) and often includes an underlying
/// cause or context.
///
/// Many variants implement `From` conversions automatically (thanks to
/// `thiserror`), so you can use the `?` operator on errors from crates like
/// `std::io` and they will be converted into `AppError`.
#[derive(Error, Debug)]
pub enum AppError {
    /// Represents an I/O error, such as failing to read or write a file.
    ///
    /// This wraps [`std::io::Error`] and provides a conversion via `From`.
    /// It occurs when file operations (opening, reading, writing, metadata)
    /// fail, or when the underlying OS returns an error.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::fs::File;
    /// use comment_remover::error::AppError;
    ///
    /// fn read_file() -> Result<(), AppError> {
    ///     let _file = File::open("nonexistent.txt")?; // automatically converts io::Error -> AppError::Io
    ///     Ok(())
    /// }
    /// ```
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// Errors originating from tree‑sitter operations.
    ///
    /// This variant is used when the tree‑sitter library fails to load a
    /// language, parse a source file, or execute a query. The inner string
    /// provides a descriptive error message. This typically indicates a
    /// problem with the grammar or the input itself.
    #[error("Tree-sitter error: {0}")]
    TreeSitter(String),

    /// The specified programming language is not supported or not enabled.
    ///
    /// This can happen if the user requests a language that was not compiled
    /// into the binary (due to feature flags), if the language name is unknown,
    /// or if automatic detection from a file extension fails.
    ///
    /// # Example
    ///
    /// ```
    /// # use comment_remover::error::AppError;
    /// let lang = "brainfuck".to_string();
    /// let err = AppError::UnsupportedLanguage(lang);
    /// assert_eq!(err.to_string(), "Unsupported language: brainfuck");
    /// ```
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),

    /// An error related to tree‑sitter query construction or execution.
    ///
    /// This can occur when a query has invalid syntax, references non‑existent
    /// captures, or fails during matching. It usually points to a bug in the
    /// comment query for a language.
    #[error("Query error: {0}")]
    Query(String),

    /// An error during parsing with tree‑sitter.
    ///
    /// This variant is used when tree‑sitter fails to produce a syntax tree,
    /// for example because the input is not valid source code or because the
    /// parser encountered an unrecoverable error.
    #[error("Parse error: {0}")]
    Parse(String),

    /// Errors related to configuration file loading or parsing.
    ///
    /// This wraps issues like invalid TOML syntax, missing required fields,
    /// or file not found when a configuration file is expected.
    ///
    /// # Example
    ///
    /// If a TOML configuration file contains a syntax error, the `toml::de`
    /// error will be converted into this variant:
    ///
    /// ```rust
    /// # use comment_remover::error::AppError;
    /// # fn example() -> Result<(), AppError> {
    /// let bad_toml = "language = true\n"; // expecting string, got boolean
    /// let _config: toml::Value = toml::from_str(bad_toml).map_err(|e| AppError::Config(e.to_string()))?;
    /// # Ok(())
    /// # }
    /// ```
    #[error("Configuration error: {0}")]
    Config(String),

    /// No files were found to process.
    ///
    /// This can be returned when the input paths (files, directories, or globs)
    /// match zero files, or when stdin is used but no language is specified.
    /// In parallel processing with `--force`, this may also be used to indicate
    /// that some files failed (as a placeholder error).
    #[error("No files processed")]
    NoFiles,

    /// A directory was provided as input without the `--recursive` flag.
    ///
    /// The inner string contains the path of the directory that caused the error.
    #[error("Directory not allowed without --recursive: {0}")]
    DirectoryNotAllowed(String),

    /// Reading from standard input requires an explicit language selection.
    ///
    /// This error is returned when the user pipes data via stdin but does not
    /// provide the `--language` (`-l`) flag, so the tool does not know which
    /// language's comments to remove.
    #[error("Language must be specified for stdin input (use -l/--language)")]
    StdinLanguageRequired,

    /// Attempted to write multiple output files to stdout without `--in-place`.
    ///
    /// When processing multiple input files, the tool can only write to stdout
    /// if the `--in-place` flag is also used (which writes each file back to
    /// itself). Without it, writing all outputs interleaved to stdout is not
    /// supported.
    #[error("Cannot output multiple files to stdout without --in-place")]
    MultipleFilesStdout,
}

/// A convenience result type alias using [`AppError`] as the error variant.
///
/// This is the standard result type used throughout the application.
///
/// # Example
///
/// ```
/// use comment_remover::error::Result;
///
/// fn always_ok() -> Result<()> {
///     Ok(())
/// }
/// ```
pub type Result<T> = std::result::Result<T, AppError>;
