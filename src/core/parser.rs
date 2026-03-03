//! Tree-sitter parser management with thread-local caching.
//!
//! This module provides a thread-local cache for tree-sitter parsers,
//! allowing reuse of a parser for the same language across multiple parse
//! operations within the same thread. This avoids the overhead of repeatedly
//! creating and configuring parsers.
//!
//! The main entry point is [`parse`], which returns a syntax tree for the given
//! input string and language. The cache is automatically managed; you can also
//! clear it explicitly with [`clear_cache`] if needed.
//!
//! # Thread Safety
//!
//! Each thread has its own independent parser cache. This means the module is
//! safe to use in multi-threaded contexts (e.g., with Rayon) because parsers are
//! never shared across threads. The cache uses `thread_local!` and `RefCell` to
//! provide interior mutability within a thread.
//!
//! # Examples
//!
//! Basic parsing:
//! ```
//! use comment_remover::core::language::TreeSitterLanguage;
//! use comment_remover::core::parser::parse;
//!
//! let code = "fn main() { println!(\"Hello\"); }";
//! let tree = parse(code, TreeSitterLanguage::Rust).unwrap();
//! println!("Root node: {:?}", tree.root_node());
//! ```
//!
//! Clearing the cache (e.g., after many parses to free memory):
//! ```
//! use comment_remover::core::parser::clear_cache;
//! clear_cache();
//! ```

use std::cell::RefCell;
use tree_sitter::{Parser as TSParser, Tree};

use crate::core::language::TreeSitterLanguage;
use crate::error::{AppError, Result};

thread_local! {
    static PARSER_CACHE: RefCell<Option<(TreeSitterLanguage, TSParser)>> = const { RefCell::new(None) };
}

/// Creates a new tree-sitter parser configured for a specific language.
///
/// This is a private helper used by [`parse`] when a parser for the requested
/// language is not already cached.
///
/// # Arguments
///
/// * `language` - The language for which to create a parser.
///
/// # Returns
///
/// * `Ok(TSParser)` on success.
/// * `Err(AppError::TreeSitter)` if the grammar cannot be loaded (e.g., language
///   not supported or compilation error).
fn create_parser(language: TreeSitterLanguage) -> Result<TSParser> {
    let mut parser = TSParser::new();
    parser.set_language(&language.get_language()).map_err(|e| {
        AppError::TreeSitter(format!("Failed to load grammar for {:?}: {}", language, e))
    })?;
    Ok(parser)
}

/// Parses a source code string using the specified language.
///
/// This function uses a thread-local cache to reuse parsers for the same language
/// within a thread. If a parser for the given language is already cached, it is
/// reused; otherwise, a new parser is created, cached, and then used.
///
/// # Arguments
///
/// * `input` - The source code as a string slice.
/// * `language` - The programming language of the source code.
///
/// # Returns
///
/// * `Ok(Tree)` containing the syntax tree if parsing succeeds.
/// * `Err(AppError::Parse)` if parsing fails (e.g., due to syntax errors).
/// * `Err(AppError::TreeSitter)` if the grammar cannot be loaded (should not
///   happen if the language is supported, but possible due to internal errors).
///
/// # Examples
///
/// ```
/// use comment_remover::core::language::TreeSitterLanguage;
/// use comment_remover::core::parser::parse;
///
/// let source = "x = 1 + 2";
/// let tree = parse(source, TreeSitterLanguage::Python).unwrap();
/// assert!(tree.root_node().has_error() == false);
/// ```
pub fn parse(input: &str, language: TreeSitterLanguage) -> Result<Tree> {
    PARSER_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        let parser = match cache.as_mut() {
            Some((lang, parser)) if *lang == language => parser,
            _ => {
                let parser = create_parser(language)?;
                *cache = Some((language, parser));
                &mut cache.as_mut().unwrap().1
            }
        };
        parser
            .parse(input, None)
            .ok_or_else(|| AppError::Parse("Failed to parse input".to_string()))
    })
}

/// Clears the thread-local parser cache.
///
/// This drops all cached parsers for the current thread, freeing any associated
/// memory. It is useful if you are done parsing and want to release resources,
/// or if you need to force a fresh parser (e.g., after a language change).
///
/// # Examples
///
/// ```
/// use comment_remover::core::parser::clear_cache;
///
/// // After many parses, clear the cache
/// clear_cache();
/// ```
pub fn clear_cache() {
    PARSER_CACHE.with(|cache| *cache.borrow_mut() = None);
}