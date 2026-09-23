//! Core comment removal logic using tree-sitter.
//!
//! This module provides the [`CommentRemover`] struct, which is the primary tool
//! for stripping comments from source code. It leverages tree-sitter to parse
//! the code, identify comment nodes via language-specific queries, and produce
//! a cleaned version of the input.
//!
//! The remover can optionally collapse consecutive blank lines to a specified
//! maximum using the `collapse` parameter. This helps reduce excessive vertical
//! whitespace that may be left after removing comments.
//!
//! # Examples
//!
//! Removing comments from a string:
//! ```
//! use comment_remover::core::{TreeSitterLanguage, CommentRemover};
//!
//! let code = r#"
//! // This is a comment
//! fn main() {
//!     println!("Hello");
//! }
//! "#;
//!
//! let remover = CommentRemover::new(TreeSitterLanguage::Rust, None);
//! let cleaned = remover.process_str(code).unwrap();
//! assert!(!cleaned.contains("// This is a comment"));
//! ```
//!
//! Processing a file with whitespace collapsing (max 1 blank line):
//! ```
//! use comment_remover::core::{TreeSitterLanguage, CommentRemover};
//! use std::path::Path;
//!
//! let remover = CommentRemover::new(TreeSitterLanguage::Python, Some(1));
//! // let result = remover.process_file(Path::new("script.py")).unwrap();
//! ```

use std::fs;
use std::path::Path;
use regex::Regex;
use tree_sitter::{Query, QueryCursor, StreamingIterator};

use crate::core::language::{COMMENT_QUERIES, TreeSitterLanguage};
use crate::core::parser;
use crate::core::whitespace::collapse_whitespace;
use crate::error::{AppError, Result, io_error};

/// Comments matching any of these patterns are always kept, on top of
/// whatever `--keep-pattern`/config supplies. These are lexically comments
/// but carry real compiler/tool semantics in their respective ecosystems --
/// removing them silently changes program or build behavior, not just
/// documentation. A tree-sitter comment query has no way to tell the two
/// apart on its own, so this list exists to cover the common cases without
/// requiring every caller to know and pass them explicitly.
pub const DEFAULT_KEEP_PATTERNS: &[&str] = &[
    r"@ts-expect-error",
    r"@ts-ignore",
    r"@ts-nocheck",
    r"eslint-disable",
    r"@vitest-environment",
    r"@vite-ignore",
    r"webpackChunkName",
    r"webpackPrefetch",
    r"webpackPreload",
    r"istanbul ignore",
    r"prettier-ignore",
    r"^///\s*<reference\b",
    r"^//\s*#!\[allow\(",
    r"^#\[allow\(",
    r"^#!\[allow\(",
    r"\bSAFETY:",
    r"deno-lint",
    r"pragma\s+solidity",
    r"^#\s*type:\s*ignore",
    r"^#\s*noqa\b",
    r"^#\s*pylint:\s*disable",
];

/// Compiles `DEFAULT_KEEP_PATTERNS` plus any caller-supplied extra patterns
/// into a single `Regex` list. Returns an error string (not `AppError`, to
/// keep this module free of a dependency edge back onto CLI-facing error
/// variants) naming the first pattern that fails to compile, so a typo in
/// `--keep-pattern` is reported clearly instead of silently doing nothing.
///
/// # Examples
/// ```
/// use comment_remover::core::remover::{compile_keep_patterns, DEFAULT_KEEP_PATTERNS};
///
/// let patterns = compile_keep_patterns(&["TODO".to_string()]).unwrap();
/// assert_eq!(patterns.len(), DEFAULT_KEEP_PATTERNS.len() + 1);
/// assert!(patterns.iter().any(|p| p.is_match("@ts-expect-error")));
/// assert!(patterns.iter().any(|p| p.is_match("TODO: fix this")));
///
/// assert!(compile_keep_patterns(&["(".to_string()]).is_err());
/// ```
pub fn compile_keep_patterns(extra: &[String]) -> std::result::Result<Vec<Regex>, String> {
    DEFAULT_KEEP_PATTERNS
        .iter()
        .map(|p| (*p).to_string())
        .chain(extra.iter().cloned())
        .map(|p| Regex::new(&p).map_err(|e| format!("invalid --keep-pattern {p:?}: {e}")))
        .collect()
}

/// A comment remover configured for a specific language and optional whitespace
/// collapsing.
///
/// The remover uses a tree-sitter parser to analyze the source code and locate
/// comments according to the language's grammar. It then reconstructs the code
/// without the comment text, preserving newline characters from the original
/// comments so that line numbers remain roughly the same (unless whitespace
/// collapsing is applied afterward).
///
/// # Example
/// ```
/// use comment_remover::core::{TreeSitterLanguage, CommentRemover};
///
/// let remover = CommentRemover::new(TreeSitterLanguage::JavaScript, Some(2));
/// ```
#[derive(Debug, Clone)]
pub struct CommentRemover {
    language: TreeSitterLanguage,
    collapse: Option<usize>,
    keep_patterns: Vec<Regex>,
}

impl CommentRemover {
    /// Same as [`new`](Self::new), but comments matching any of
    /// `keep_patterns` are left in place instead of removed. Callers
    /// wanting the built-in directive-comment protections
    /// (`@ts-expect-error`, `eslint-disable`, `/// <reference>`, ...) should
    /// build `keep_patterns` via [`compile_keep_patterns`], not duplicate
    /// [`DEFAULT_KEEP_PATTERNS`] by hand.
    pub fn with_keep_patterns(
        language: TreeSitterLanguage,
        collapse: Option<usize>,
        keep_patterns: Vec<Regex>,
    ) -> Self {
        Self { language, collapse, keep_patterns }
    }
    /// Creates a new `CommentRemover` for the given language.
    ///
    /// # Arguments
    ///
    /// * `language` - The programming language of the input source code.
    /// * `collapse` - If `Some(n)`, sequences of blank lines longer than `n`
    ///   will be reduced to at most `n` consecutive newlines. If `None`, no
    ///   whitespace collapsing is performed.
    ///
    /// # Returns
    ///
    /// A new `CommentRemover` instance.
    ///
    /// # Examples
    /// ```
    /// use comment_remover::core::{TreeSitterLanguage, CommentRemover};
    ///
    /// let remover = CommentRemover::new(TreeSitterLanguage::Rust, Some(1));
    /// ```
    pub fn new(language: TreeSitterLanguage, collapse: Option<usize>) -> Self {
        Self { language, collapse, keep_patterns: Vec::new() }
    }

    /// Removes comments from a source code string.
    ///
    /// This method parses the input, finds all comment nodes using the language's
    /// predefined query, and rebuilds the string without the comment text. Only
    /// the newline characters from comments are retained to preserve line structure.
    /// If `collapse` is set, the resulting string will also have consecutive blank
    /// lines reduced.
    ///
    /// # Arguments
    ///
    /// * `input` - The source code as a string slice.
    ///
    /// # Returns
    ///
    /// * `Ok(String)` containing the cleaned source code.
    /// * `Err(AppError::Parse)` if the input cannot be parsed.
    /// * `Err(AppError::Query)` if the comment query for the language is invalid
    ///   (should not happen in normal use).
    /// * `Err(AppError::UnsupportedLanguage)` if no comment query is defined for
    ///   the language (should not happen if the language is correctly initialized).
    ///
    /// # Examples
    /// ```
    /// use comment_remover::core::{TreeSitterLanguage, CommentRemover};
    ///
    /// let remover = CommentRemover::new(TreeSitterLanguage::Python, None);
    /// let code = "# a comment\nprint('hello')";
    /// let cleaned = remover.process_str(code).unwrap();
    /// assert_eq!(cleaned, "\nprint('hello')");
    /// ```
    pub fn process_str(&self, input: &str) -> Result<String> {
        let tree = parser::parse(input, self.language)?;
        let query_str = COMMENT_QUERIES
            .get(&self.language)
            .ok_or_else(|| AppError::UnsupportedLanguage(format!("{:?}", self.language)))?;

        let ts_lang = self.language.get_language();
        let query = Query::new(&ts_lang, query_str)
            .map_err(|e| AppError::Query(format!("Failed to create query: {}", e)))?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), input.as_bytes());

        let mut comment_ranges: Vec<std::ops::Range<usize>> = Vec::new();
        while let Some(m) = matches.next() {
            for capture in m.captures {
                let range = capture.node.byte_range();
                let text = &input[range.clone()];
                if self.keep_patterns.iter().any(|p| p.is_match(text)) {
                    continue;
                }
                comment_ranges.push(range);
            }
        }

        comment_ranges.sort_by_key(|r| r.start);

        let mut result = String::with_capacity(input.len());
        let mut last_pos = 0;
        for range in comment_ranges {
            result.push_str(&input[last_pos..range.start]);

            // Preserve only newlines from the comment
            result.extend(input[range.clone()].chars().filter(|&c| c == '\n'));
            last_pos = range.end;
        }
        result.push_str(&input[last_pos..]);

        if let Some(max) = self.collapse {
            result = collapse_whitespace(&result, max);
        }

        Ok(result)
    }

    /// Removes comments from a file.
    ///
    /// This is a convenience method that reads the file at `path` and then calls
    /// [`process_str`](Self::process_str) on its contents.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to process.
    ///
    /// # Returns
    ///
    /// * `Ok(String)` containing the cleaned source code.
    /// * `Err(AppError::Io)` if the file cannot be read.
    /// * Other errors as described in [`process_str`](Self::process_str).
    ///
    /// # Examples
    /// ```
    /// use comment_remover::core::{TreeSitterLanguage, CommentRemover};
    /// use std::path::Path;
    ///
    /// let remover = CommentRemover::new(TreeSitterLanguage::Rust, None);
    /// // let cleaned = remover.process_file(Path::new("src/lib.rs")).unwrap();
    /// ```
    pub fn process_file(&self, path: &Path) -> Result<String> {
        let content = fs::read_to_string(path).map_err(|e| io_error(path, e))?;
        self.process_str(&content)
    }
}

#[cfg(all(test, feature = "rust-lang"))]
mod keep_pattern_tests {
    use super::*;

    #[test]
    fn plain_comment_is_removed_directive_comment_is_kept() {
        let input = "// plain noise\n// SAFETY: caller upholds the invariant\nfn f() {}";
        let patterns = compile_keep_patterns(&[]).unwrap();
        let remover = CommentRemover::with_keep_patterns(TreeSitterLanguage::Rust, None, patterns);
        let out = remover.process_str(input).unwrap();
        assert!(!out.contains("plain noise"), "non-matching comment must still be removed: {out:?}");
        assert!(out.contains("SAFETY: caller upholds the invariant"), "SAFETY comment must survive: {out:?}");
    }

    #[test]
    fn no_keep_patterns_removes_everything_same_as_before() {
        let input = "// SAFETY: this would normally be protected\nfn f() {}";
        let remover = CommentRemover::new(TreeSitterLanguage::Rust, None);
        let out = remover.process_str(input).unwrap();
        assert!(!out.contains("SAFETY"), "with no keep_patterns, behavior must match the pre-existing new(): {out:?}");
    }

    #[test]
    fn extra_pattern_protects_a_project_specific_marker() {
        let input = "// KEEP-ME: project marker\n// drop this one\nfn f() {}";
        let patterns = compile_keep_patterns(&["KEEP-ME".to_string()]).unwrap();
        let remover = CommentRemover::with_keep_patterns(TreeSitterLanguage::Rust, None, patterns);
        let out = remover.process_str(input).unwrap();
        assert!(out.contains("KEEP-ME: project marker"), "custom --keep-pattern must be honored: {out:?}");
        assert!(!out.contains("drop this one"), "unmatched comment must still be removed: {out:?}");
    }

    #[test]
    fn invalid_pattern_reports_an_error_not_a_silent_skip() {
        assert!(compile_keep_patterns(&["(unclosed".to_string()]).is_err());
    }
}