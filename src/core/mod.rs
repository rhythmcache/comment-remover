//! # Core Engine
//!
//! The `core` module provides the main comment-removal engine built on top of
//! [tree-sitter]. It is responsible for:
//!
//! - Detecting programming language support
//! - Parsing source code into syntax trees
//! - Extracting comment nodes using language-specific queries
//! - Removing comment ranges safely without corrupting code
//! - Optionally normalizing whitespace
//!
//! This module is designed to be independent from CLI or I/O layers.
//! It operates purely on strings or file paths.
//!
//! ## Architecture Overview
//!
//! The module is separated into four focused submodules:
//!
//! - [`language`] — Defines [`TreeSitterLanguage`] and language detection logic.
//! - [`parser`] — Provides a cached tree-sitter parser and parsing utilities.
//! - [`remover`] — Implements [`CommentRemover`], the main processing engine.
//! - [`whitespace`] — Utilities for whitespace normalization.
//!
//! All key types and functions are re-exported at the `core` level
//! for ergonomic usage.
//!
//! ---
//!
//! ## Design Principles
//!
//! - **Syntax-aware removal** — Comments are removed using AST queries,
//!   not naive string matching.
//! - **Language-safe** — Each supported language defines its own
//!   comment query pattern.
//! - **Thread-local parser caching** — Improves performance for
//!   repeated parsing operations.
//! - **Deterministic output** — Removal preserves non-comment syntax.
//! - **Optional whitespace collapsing** — Consecutive blank lines
//!   can be normalized.
//!
//! ---
//!
//! ## Re-exports
//!
//! This module re-exports the most commonly used items:
//!
//! - [`TreeSitterLanguage`]
//! - [`parse`]
//! - [`clear_cache`]
//! - [`CommentRemover`]
//! - [`collapse_whitespace`]
//!
//! ---
//!
//! ## Example: Remove Comments from a String
//!
//! ```
//! use comment_remover::core::{TreeSitterLanguage, CommentRemover};
//!
//! let source = r#"
//! fn main() {
//!     // remove this
//!     println!("hello");
//! }
//! "#;
//!
//! let remover = CommentRemover::new(TreeSitterLanguage::Rust, None);
//! let output = remover.process_str(source).unwrap();
//!
//! assert!(!output.contains("// remove this"));
//! ```
//!
//! ---
//!
//! ## Example: Remove Comments from a File
//!
//! ```no_run
//! use comment_remover::core::{TreeSitterLanguage, CommentRemover};
//! use std::path::Path;
//!
//! let remover = CommentRemover::new(TreeSitterLanguage::Python, Some(1));
//! let result = remover.process_file(Path::new("script.py")).unwrap();
//! ```
//!
//! ---
//!
//! ## Performance Notes
//!
//! - Parsing uses a thread-local cached `tree_sitter::Parser`.
//! - Queries are compiled per-language.
//! - Memory usage scales with input size and syntax tree depth.
//!
//! ---
//!
//! ## Safety Considerations
//!
//! - Only comment nodes defined by language queries are removed.
//! - String literals and doc strings are preserved unless explicitly
//!   defined as comments in the grammar.
//! - Behavior depends on correctness of the upstream tree-sitter grammar.
//!
//! ---
//!
//! ## Supported Languages
//!
//! See [`TreeSitterLanguage`] for the current list of supported languages
//! and associated feature flags.
//!
//! ---
//!
//! ## When to Use
//!
//! Use this module when you need:
//!
//! - Preprocessing before static analysis
//! - Code normalization pipelines
//! - Source code compression
//! - Removing comments before diff or comparison
//!
//! Avoid using it for:
//!
//! - Minification beyond comment removal
//! - Formatting (use a formatter instead)
//!
//! ---
//!
//! [tree-sitter]: https://tree-sitter.github.io/tree-sitter/

/// Core engine modules for comment removal.
pub mod language; // Language detection & comment query strings
pub mod parser; // Thread-local tree-sitter parser utilities
pub mod remover; // Main comment removal engine
pub mod whitespace; // Utilities for collapsing extra blank lines

// Re-export key items for convenience
pub use language::*;
pub use parser::*;
pub use remover::*;
pub use whitespace::*;
