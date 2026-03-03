//! Whitespace normalization utilities.
//!
//! This module provides functions for cleaning up whitespace in source code,
//! primarily by collapsing consecutive blank lines to a specified maximum.
//! This is useful after removing comments, which can leave large gaps of
//! empty lines that disrupt code readability.
//!
//! The main function is [`collapse_whitespace`], which processes a string
//! line by line and reduces sequences of empty lines to at most `max_newlines`
//! consecutive newlines.

/// Collapses consecutive blank lines in a string to at most `max_newlines`.
///
/// This function processes the input string line by line, tracking sequences
/// of lines that contain only whitespace (or are completely empty). When a
/// non‑empty line is encountered (or the end of the string is reached), the
/// accumulated blank lines are replaced by at most `max_newlines` newline
/// characters. The original non‑empty lines are preserved unchanged.
///
/// The function preserves the trailing newline of the input: if the original
/// string ends with a newline, the result will also end with a newline
/// (unless the entire string consists of blank lines and they are collapsed
/// to fewer newlines).
///
/// # Special behavior
///
/// - If `max_newlines` is `usize::MAX`, the input is returned as‑is, without
///   any collapsing. This can be used to bypass whitespace normalization
///   efficiently.
/// - A line is considered "empty" if `line.trim().is_empty()` returns `true`.
///   This includes lines that contain only spaces or tabs.
/// - Blank lines at the very beginning or end of the string are handled
///   correctly: leading blank lines are preserved up to the limit, and
///   trailing blank lines are also preserved up to the limit.
///
/// # Arguments
///
/// * `input` - The string to process.
/// * `max_newlines` - The maximum number of consecutive newlines (blank lines)
///   to allow. If the actual number of consecutive blank lines exceeds this,
///   they are reduced to exactly `max_newlines` newlines. If it is less than
///   or equal, all blank lines are kept.
///
/// # Returns
///
/// A new `String` with the same content as `input`, except that runs of blank
/// lines are shortened to at most `max_newlines` newlines.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use comment_remover::core::whitespace::collapse_whitespace;
///
/// let text = "line1\n\n\n\nline2";
/// let collapsed = collapse_whitespace(text, 2);
/// assert_eq!(collapsed, "line1\n\n\nline2"); // four blank lines become two
/// ```
///
/// Leading and trailing blank lines:
///
/// ```
/// # use comment_remover::core::whitespace::collapse_whitespace;
/// let text = "\n\n\nHello\n\nWorld\n\n\n";
/// let collapsed = collapse_whitespace(text, 1);
/// // leading: 3 → 1 newline, trailing: 3 → 1 newline
/// assert_eq!(collapsed, "\nHello\n\nWorld\n");
/// ```
///
/// When `max_newlines` is 0, all blank lines are removed:
///
/// ```
/// # use comment_remover::core::whitespace::collapse_whitespace;
/// let text = "a\n\n\nb\n\nc";
/// let collapsed = collapse_whitespace(text, 0);
/// assert_eq!(collapsed, "a\nb\nc");
/// ```
///
/// Using `usize::MAX` disables collapsing:
///
/// ```
/// # use comment_remover::core::whitespace::collapse_whitespace;
/// let text = "a\n\n\nb";
/// let collapsed = collapse_whitespace(text, usize::MAX);
/// assert_eq!(collapsed, text);
/// ```
///
/// Lines containing only spaces or tabs are considered empty:
///
/// ```
/// # use comment_remover::core::whitespace::collapse_whitespace;
/// let text = "x\n   \n\t\n   \ny";
/// let collapsed = collapse_whitespace(text, 1);
/// assert_eq!(collapsed, "x\n\ny"); // three blank lines become one
/// ```
pub fn collapse_whitespace(input: &str, max_newlines: usize) -> String {
    if max_newlines == usize::MAX {
        return input.to_string();
    }

    let mut result = String::with_capacity(input.len());
    let mut empty_count = 0;

    for line in input.split_inclusive('\n') {
        if line.trim().is_empty() {
            empty_count += 1;
        } else {
            if empty_count > 0 {
                let keep = if empty_count <= max_newlines {
                    empty_count
                } else {
                    max_newlines
                };
                for _ in 0..keep {
                    result.push('\n');
                }
                empty_count = 0;
            }
            result.push_str(line);
        }
    }

    if empty_count > 0 {
        let keep = if empty_count <= max_newlines {
            empty_count
        } else {
            max_newlines
        };
        for _ in 0..keep {
            result.push('\n');
        }
    } else if input.ends_with('\n') && !result.ends_with('\n') {
        result.push('\n');
    }

    result
}