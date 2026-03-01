//! A fast, accurate comment remover for multiple programming languages, powered by tree-sitter.
//!
//! This crate provides both a library API and a command-line interface to strip comments
//! from source code files. It uses [tree-sitter](https://tree-sitter.github.io/) to parse
//! each file according to its language grammar, ensuring that only actual comments are
//! removed – not strings, regular expressions, or other syntactic elements that may look
//! like comments.
//!
//! # Features
//!
//! * Support for many languages (each gated by a feature flag).
//! * Automatic language detection from file extensions.
//! * Whitespace collapsing: reduce consecutive blank lines to a configurable maximum.
//! * Parallel processing using Rayon.
//! * Dry-run and diff modes to preview changes.
//! * JSON output for integration with other tools.
//! * Configurable via TOML file and command-line arguments.
//!
//! # Modules
//!
//! * [`cli`] – Command-line argument parsing and main driver.
//! * [`config`] – Configuration file loading and merging with CLI options.
//! * [`core`] – The comment removal engine: language definitions, parser, remover, whitespace utilities.
//! * [`error`] – Error types and `Result` alias.
//! * [`io`] – File I/O helpers and file collection.
//!
//! # Example (library usage)
//!
//! ```rust
//! use comment_remover::core::{TreeSitterLanguage, CommentRemover};
//!
//! let code = r#"
//! fn main() {
//!     // A comment to remove
//!     println!("Hello, world!"); // trailing comment
//! }
//! "#;
//!
//! let remover = CommentRemover::new(TreeSitterLanguage::Rust, None);
//! let cleaned = remover.process_str(code).unwrap();
//! assert!(!cleaned.contains("// A comment to remove"));
//! assert!(!cleaned.contains("// trailing comment"));
//! ```
//!
//! # Example (CLI)
//!
//! ```bash
//! comment-remover file.rs --in-place
//! comment-remover src/ --recursive --output-dir out/
//! cat script.py | comment-remover -l python
//! ```
//!
//! # Feature flags
//!
//! Each language is optional and can be enabled with a feature (e.g., `python`, `rust-lang`).
//! By default, no languages are enabled; you must select the ones you need.
//!
//! * `bash`
//! * `c`
//! * `c-sharp`
//! * `cpp`
//! * `css`
//! * `go`
//! * `haskell`
//! * `html`
//! * `java`
//! * `javascript`
//! * `lua`
//! * `php`
//! * `python`
//! * `ruby`
//! * `rust-lang`
//! * `scala`
//! * `swift`
//! * `typescript`
//! * `sql`
//! * `perl`
//! * `r`
//! * `dart`
//! * `elixir`
//! * `toml`
//! * `ini`
//!
//! Additional features:
//! * `default` – none (you must opt-in to languages).
//! * `all` – enables all languages (convenience for building a full binary).

#![warn(missing_docs)]

pub mod cli;
pub mod config;
pub mod core;
pub mod error;
pub mod io;