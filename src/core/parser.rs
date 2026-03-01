//! Thread-local parser management for tree‑sitter.
//!
//! This module provides a performant way to obtain a tree‑sitter parser
//! configured for a specific language. It maintains a thread‑local cache
//! of the most recently used parser, avoiding the overhead of creating a
//! new parser for every file when processing many files of the same language.
//!
//! The primary entry point is [`parse`], which takes a source string and a
//! language, and returns a syntax tree. The parser is automatically reused
//! if the language matches the previous call in the same thread.
//!
//! # Thread Safety
//!
//! Tree‑sitter parsers are not `Send` or `Sync`, so they cannot be shared
//! across threads. This module solves that by storing the parser in
//! thread‑local storage (`std::cell::RefCell`). Each thread gets its own
//! independent cache. This is ideal for multi‑threaded scenarios where each
//! thread processes its own set of files (e.g., using Rayon or a thread pool).
//!
//! # Example
//!
//! ```
//! use comment_remover::core::language::TreeSitterLanguage;
//! use comment_remover::core::parser::parse;
//!
//! # #[cfg(feature = "rust-lang")]
//! # {
//! let source = "fn main() { println!(\"Hello\"); }";
//! let tree = parse(source, TreeSitterLanguage::Rust).expect("parsing failed");
//! assert!(!tree.root_node().has_error());
//! # }
//! ```

use std::cell::RefCell;
use tree_sitter::Parser as TSParser;
use tree_sitter::Tree;

use crate::core::language::TreeSitterLanguage;
use crate::error::{AppError, Result};

thread_local! {
    static PARSER_CACHE: RefCell<Option<(TreeSitterLanguage, TSParser)>> =
        const { RefCell::new(None) };
}

/// Parses a source string using the specified language.
///
/// This function retrieves a tree‑sitter parser from the thread‑local cache,
/// reconfiguring it if the language has changed since the last call. It then
/// parses the input and returns the resulting syntax tree.
///
/// # Arguments
///
/// * `input` – The source code to parse, as a string slice.
/// * `language` – The language to use for parsing. The corresponding feature
///   must be enabled in the build.
///
/// # Returns
///
/// A [`tree_sitter::Tree`] representing the parsed syntax tree.
///
/// # Errors
///
/// Returns an error in the following situations:
///
/// * [`AppError::TreeSitter`] – if the tree‑sitter grammar for the requested
///   language cannot be loaded (this should not happen if the language variant
///   exists, because the grammar is statically linked).
/// * [`AppError::Parse`] – if parsing fails (e.g., the input contains syntax
///   errors that prevent tree‑sitter from building a tree).
///
/// # Example
///
/// ```
/// use comment_remover::core::language::TreeSitterLanguage;
/// use comment_remover::core::parser::parse;
///
/// # #[cfg(feature = "python")]
/// # {
/// let code = "def hello(): print('world')";
/// let tree = parse(code, TreeSitterLanguage::Python)?;
/// assert!(!tree.root_node().has_error());
/// # }
/// # Ok::<_, comment_remover::error::AppError>(())
/// ```
pub fn parse(input: &str, language: TreeSitterLanguage) -> Result<Tree> {
    PARSER_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();

        // Get or create a parser for the requested language
        let parser = if let Some((cached_lang, parser)) = cache.as_mut() {
            if *cached_lang == language {
                parser
            } else {
                // Language changed, need a new parser
                let mut new_parser = TSParser::new();
                new_parser
                    .set_language(&language.get_language())
                    .map_err(|e| AppError::TreeSitter(format!("Failed to load grammar: {}", e)))?;
                *cache = Some((language, new_parser));
                &mut cache.as_mut().unwrap().1
            }
        } else {
            let mut new_parser = TSParser::new();
            new_parser
                .set_language(&language.get_language())
                .map_err(|e| AppError::TreeSitter(format!("Failed to load grammar: {}", e)))?;
            let _ = cache.insert((language, new_parser));
            &mut cache.as_mut().unwrap().1
        };

        parser
            .parse(input, None)
            .ok_or_else(|| AppError::Parse("Failed to parse input".to_string()))
    })
}

/// Clears the thread‑local parser cache.
///
/// After calling this function, the next call to [`parse`] in the same thread
/// will create a fresh parser instead of reusing any previous one. This can be
/// useful in tests to ensure a clean state, or if the parser might be in an
/// inconsistent state (though that should not normally happen).
///
/// # Example
///
/// ```
/// use comment_remover::core::parser::clear_cache;
///
/// clear_cache(); // Next parse will create a new parser
/// ```
pub fn clear_cache() {
    PARSER_CACHE.with(|cache| {
        *cache.borrow_mut() = None;
    });
}
