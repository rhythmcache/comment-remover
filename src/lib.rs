//! # comment-remover
//!
//! A library and command-line tool for removing comments from source code
//! using the tree‑sitter parsing library.
//!
//! This crate provides both a reusable library (the `core` module) and a
//! command‑line interface (the `cli` module). The core logic is language‑
//! agnostic and driven by tree‑sitter queries; adding support for a new
//! language simply requires a query that matches its comment nodes.
//!
//! ## Library Usage
//!
//! The main entry point for programmatic use is [`core::CommentRemover`].
//! You can create an instance for a specific language and optional whitespace
//! collapsing, then process strings or files:
//!
//! ```rust
//! use comment_remover::core::{CommentRemover, TreeSitterLanguage};
//!
//! # #[cfg(feature = "rust-lang")]
//! # {
//! let remover = CommentRemover::new(TreeSitterLanguage::Rust, Some(1));
//! let source = "// a comment\nfn main() {}";
//! let cleaned = remover.process_str(source).unwrap();
//! assert_eq!(cleaned, "\nfn main() {}");
//! # }
//! ```
//!
//! ## Feature Flags
//!
//! Each language is gated by a separate Cargo feature, allowing you to
//! build a minimal binary with only the languages you need. The `default`
//! feature enables a reasonable set of languages (e.g., Rust, Python,
//! JavaScript). To enable all languages, use the `all-languages` feature.
//!
//! ## Modules
//!
//! - [`cli`]: Command‑line interface parsing and execution.
//! - [`config`]: Configuration file loading and merging.
//! - [`core`]: Core comment‑removal engine, including language definitions,
//!   thread‑local parser caching, and whitespace collapsing.
//! - [`error`]: Centralised error types and results.
//! - [`io`]: File and directory I/O utilities.
//!
//! ## Re-exports
//!
//! For convenience, the main error types are re‑exported at the crate root:
//!
//! - [`AppError`] – The primary error enum.
//! - [`Result`]  – A type alias for `std::result::Result<T, AppError>`.
//!
//! These can be used directly via `use comment_remover::{AppError, Result};`.
//!
//! For more details, see each module’s documentation.

#![warn(missing_docs)]

pub mod cli;
pub mod config;
pub mod core;
pub mod error;
pub mod io;

pub use crate::error::{AppError, Result};