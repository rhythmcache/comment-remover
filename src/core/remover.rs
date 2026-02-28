//! Core comment removal logic using tree‑sitter queries.
//!
//! This module provides the main functionality for stripping comments from
//! source code. The central type is [`CommentRemover`], which holds a
//! language and an optional whitespace‑collapse setting, and offers methods
//! to process strings or entire files.
//!
//! # How It Works
//!
//! 1. Parse the input using a thread‑local tree‑sitter parser (see
//!    [`crate::core::parser`]).
//! 2. Retrieve the comment query for the given language from the
//!    [`COMMENT_QUERIES`](crate::core::language::COMMENT_QUERIES) map.
//! 3. Execute the query to obtain all byte ranges that correspond to comments.
//! 4. Reconstruct the source code by omitting those ranges, but preserving the
//!    newline characters that were inside comments (so line numbers stay
//!    consistent).
//! 5. Optionally collapse runs of consecutive blank lines to a specified maximum.
//!
//! # Example
//!
//! ```
//! use comment_remover::core::language::TreeSitterLanguage;
//! use comment_remover::core::remover::CommentRemover;
//!
//! # #[cfg(feature = "rust-lang")]
//! # {
//! let remover = CommentRemover::new(TreeSitterLanguage::Rust, Some(1));
//! let source = r#"
//! // This is a comment
//! fn main() {
//!     println!("Hello, world!"); // inline comment
//! }
//! "#;
//!
//! let cleaned = remover.process_str(source).unwrap();
//! assert!(!cleaned.contains("comment"));
//! assert!(cleaned.contains("println!"));
//! # }
//! ```

use tree_sitter::{Query, QueryCursor, StreamingIterator};
use std::fs;
use std::path::Path;

use crate::core::language::{COMMENT_QUERIES, TreeSitterLanguage};
use crate::core::parser;
use crate::core::whitespace::collapse_whitespace;
use crate::error::{AppError, Result};

/// A configured comment remover for a specific language.
///
/// This struct holds the language and an optional whitespace‑collapse policy.
/// It is cheap to clone and can be reused for many files.
///
/// # Example
///
/// ```
/// # use comment_remover::core::language::TreeSitterLanguage;
/// # use comment_remover::core::remover::CommentRemover;
/// let remover = CommentRemover::new(TreeSitterLanguage::Python, None);
/// ```
#[derive(Debug, Clone)]
pub struct CommentRemover {
    /// The programming language to use for parsing and comment detection.
    language: TreeSitterLanguage,

    /// If `Some(n)`, collapses sequences of more than `n` consecutive blank
    /// lines to exactly `n` blank lines.
    collapse: Option<usize>,
}

impl CommentRemover {
    /// Creates a new `CommentRemover` for the specified language.
    ///
    /// # Arguments
    ///
    /// * `language` – The language to use. The corresponding Cargo feature
    ///   must be enabled.
    /// * `collapse` – If `Some(max)`, blank lines will be collapsed so that
    ///   no more than `max` consecutive blank lines appear. Use `None` to
    ///   disable blank‑line collapsing.
    ///
    /// # Example
    ///
    /// ```
    /// # use comment_remover::core::language::TreeSitterLanguage;
    /// # use comment_remover::core::remover::CommentRemover;
    /// // Create a remover for Rust with blank lines collapsed to at most 2.
    /// let remover = CommentRemover::new(TreeSitterLanguage::Rust, Some(2));
    /// ```
    pub fn new(language: TreeSitterLanguage, collapse: Option<usize>) -> Self {
        Self { language, collapse }
    }

    /// Removes comments from a string and returns the cleaned source.
    ///
    /// The input is parsed with tree‑sitter, all comment nodes are located,
    /// and then removed while preserving newlines. If a collapse limit was
    /// specified during construction, consecutive blank lines are reduced
    /// after comment removal.
    ///
    /// # Errors
    ///
    /// This function can fail for several reasons:
    ///
    /// * [`AppError::Parse`] – If tree‑sitter cannot parse the input (e.g.,
    ///   due to severe syntax errors).
    /// * [`AppError::Query`] – If the comment query for the language cannot
    ///   be compiled (this should not happen if the query is correct).
    /// * [`AppError::UnsupportedLanguage`] – If the language is not enabled
    ///   in the current build (should be caught earlier, but handled here).
    ///
    /// # Example
    ///
    /// ```
    /// # use comment_remover::core::language::TreeSitterLanguage;
    /// # use comment_remover::core::remover::CommentRemover;
    /// # #[cfg(feature = "python")]
    /// # {
    /// let remover = CommentRemover::new(TreeSitterLanguage::Python, None);
    /// let source = "# a comment\ndef foo(): pass";
    /// let cleaned = remover.process_str(source).unwrap();
    /// assert_eq!(cleaned, "\ndef foo(): pass");
    /// # }
    /// ```
    pub fn process_str(&self, input: &str) -> Result<String> {
        // Parse the input to obtain a syntax tree.
        let tree = parser::parse(input, self.language)?;

        // Retrieve the query string that matches comments for this language.
        let query_str = COMMENT_QUERIES
            .get(&self.language)
            .ok_or_else(|| AppError::UnsupportedLanguage(format!("{:?}", self.language)))?;

        // Compile the query using the tree‑sitter language.
        let ts_lang = self.language.get_language();
        let query = Query::new(&ts_lang, query_str)
            .map_err(|e| AppError::Query(format!("Failed to create query: {}", e)))?;

        // Execute the query to find all comment nodes.
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), input.as_bytes());

        let mut comment_ranges = Vec::new();
        while let Some(m) = matches.next() {
            for capture in m.captures {
                comment_ranges.push(capture.node.byte_range());
            }
        }

        // Sort the ranges by start position; they may be returned in arbitrary order.
        comment_ranges.sort_by_key(|r| r.start);

        // Rebuild the string, omitting comment ranges but preserving newlines.
        let mut result = String::with_capacity(input.len());
        let mut last_pos = 0;
        for range in comment_ranges {
            // Append the part before the comment.
            result.push_str(&input[last_pos..range.start]);
            // Preserve every newline character that was inside the comment.
            for ch in input[range.clone()].chars() {
                if ch == '\n' {
                    result.push('\n');
                }
            }
            last_pos = range.end;
        }
        // Append the remaining part after the last comment.
        result.push_str(&input[last_pos..]);

        // Optionally collapse multiple blank lines.
        if let Some(max) = self.collapse {
            result = collapse_whitespace(&result, max);
        }

        Ok(result)
    }

    /// Reads a file, removes comments from its content, and returns the cleaned source.
    ///
    /// This is a convenience wrapper around [`process_str`](Self::process_str)
    /// that first reads the file’s content into a string.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Io`] if the file cannot be read, plus any error
    /// that [`process_str`](Self::process_str) can produce.
    ///
    /// # Example
    ///
    /// ```
    /// # use comment_remover::core::language::TreeSitterLanguage;
    /// # use comment_remover::core::remover::CommentRemover;
    /// # use std::fs;
    /// # use tempfile::NamedTempFile;
    /// # #[cfg(feature = "rust-lang")]
    /// # {
    /// # let mut tmp = NamedTempFile::new().unwrap();
    /// # fs::write(tmp.path(), "// test\nfn main() {}").unwrap();
    /// let remover = CommentRemover::new(TreeSitterLanguage::Rust, None);
    /// let cleaned = remover.process_file(tmp.path()).unwrap();
    /// assert_eq!(cleaned, "\nfn main() {}");
    /// # }
    /// ```
    pub fn process_file(&self, path: &Path) -> Result<String> {
        let content = fs::read_to_string(path).map_err(AppError::Io)?;
        self.process_str(&content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create a remover for a language.
    fn remover(lang: TreeSitterLanguage) -> CommentRemover {
        CommentRemover::new(lang, None)
    }

    #[test]
    #[cfg(feature = "python")]
    fn test_python_single_line_comment() {
        let r = remover(TreeSitterLanguage::Python);
        let input = "# This is a comment\nprint('hello')";
        let result = r.process_str(input).unwrap();
        assert!(!result.contains("# This is a comment"));
        assert!(result.contains("print('hello')"));
        assert_eq!(result.matches('\n').count(), input.matches('\n').count());
    }

    #[test]
    #[cfg(feature = "python")]
    fn test_python_inline_comment() {
        let r = remover(TreeSitterLanguage::Python);
        let input = "x = 5  # set x to 5\nprint(x)";
        let result = r.process_str(input).unwrap();
        assert!(!result.contains("# set x to 5"));
        assert!(result.contains("x = 5"));
    }

    #[test]
    #[cfg(feature = "python")]
    fn test_python_preserves_strings_with_hash() {
        let r = remover(TreeSitterLanguage::Python);
        let input = "text = '# not a comment'\nprint(text)";
        let result = r.process_str(input).unwrap();
        assert!(result.contains("# not a comment"));
    }

    #[test]
    #[cfg(feature = "rust-lang")]
    fn test_rust_line_comment() {
        let r = remover(TreeSitterLanguage::Rust);
        let input = "// This is a comment\nfn main() {}";
        let result = r.process_str(input).unwrap();
        assert!(!result.contains("// This is a comment"));
        assert!(result.contains("fn main()"));
    }

    #[test]
    #[cfg(feature = "rust-lang")]
    fn test_rust_block_comment() {
        let r = remover(TreeSitterLanguage::Rust);
        let input = "/* Block comment */\nfn test() {}";
        let result = r.process_str(input).unwrap();
        assert!(!result.contains("/* Block comment */"));
        assert!(result.contains("fn test()"));
    }

    #[test]
    #[cfg(feature = "rust-lang")]
    fn test_rust_nested_block_comments() {
        let r = remover(TreeSitterLanguage::Rust);
        let input = "/* outer /* inner */ outer */\nlet x = 1;";
        let result = r.process_str(input).unwrap();
        assert!(!result.contains("outer"));
        assert!(!result.contains("inner"));
        assert!(result.contains("let x = 1"));
    }

    #[test]
    #[cfg(feature = "javascript")]
    fn test_javascript_jsdoc_removed() {
        let r = remover(TreeSitterLanguage::JavaScript);
        let input = "/**\n * JSDoc comment\n */\nfunction test() {}";
        let result = r.process_str(input).unwrap();
        assert!(!result.contains("JSDoc"));
        assert!(result.contains("function test()"));
    }

    #[test]
    #[cfg(feature = "rust-lang")]
    fn test_collapse_whitespace() {
        let r = CommentRemover::new(TreeSitterLanguage::Rust, Some(1));
        let input = "fn main() {}\n\n\n// comment\n\nfn other() {}";

        let result = r.process_str(input).unwrap();

        assert!(
            result.contains("fn main() {}\n\nfn other() {}")
                || result.contains("fn main() {}\nfn other() {}")
        );
    }

    #[test]
    #[cfg(feature = "python")]
    fn test_preserves_newlines_in_comments() {
        let r = remover(TreeSitterLanguage::Python);
        let input = "# Comment line 1\n# Comment line 2\ncode()";
        let result = r.process_str(input).unwrap();
        assert_eq!(result.matches('\n').count(), input.matches('\n').count());
    }

    #[test]
    #[cfg(feature = "javascript")]
    fn test_comment_like_strings_preserved() {
        let r = remover(TreeSitterLanguage::JavaScript);
        let input = "var url = 'http://example.com';\nvar comment = '// not a comment';";
        let result = r.process_str(input).unwrap();
        assert!(result.contains("http://example.com"));
        assert!(result.contains("// not a comment"));
    }
}