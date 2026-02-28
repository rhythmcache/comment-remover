//! Utilities for handling whitespace in source code.
//!
//! This module provides functions to manipulate whitespace, primarily for
//! collapsing consecutive blank lines after comment removal. The main function,
//! [`collapse_whitespace`], processes a string and reduces runs of empty lines
//! (or lines containing only whitespace) to a specified maximum.
//!
//! # Why Collapse Whitespace?
//!
//! After removing comments from source code, you are often left with multiple
//! consecutive blank lines. While not harmful, they can make the code harder
//! to read and increase file size unnecessarily. The whitespace collapsing
//! functionality allows you to clean up the output by limiting the number of
//! blank lines, making the result more visually pleasing and closer to typical
//! coding style.
//!
//! # How It Works
//!
//! The function splits the input into lines using [`str::lines`], which handles
//! both Unix (`\n`) and Windows (`\r\n`) line endings, normalising them to `\n`.
//! It then iterates through the lines, tracking how many consecutive lines have
//! been empty (after trimming). When a non‑empty line is encountered, it is
//! always emitted, preceded by a newline if it is not the first line. Empty
//! lines are emitted only if the current count of consecutive empty lines does
//! not exceed the user‑specified maximum. Finally, if the original input ended
//! with a newline, the output is ensured to also end with a newline.
//!
//! # Edge Cases
//!
//! - **Leading blank lines**: They are subject to the same limit as internal ones.
//! - **Trailing newline**: The function preserves whether the original ended
//!   with a newline, even if the last line is empty and gets collapsed.
//! - **Lines with only spaces/tabs**: They are considered empty.
//! - **Empty input**: Returns an empty string.
//! - **No collapsing needed**: If `max_newlines` is set to [`usize::MAX`], the
//!   input is returned unchanged (except for a possible allocation).
//!
//! # Integration
//!
//! This module is typically used after [`crate::core::remover::CommentRemover`]
//! has stripped comments, passing the result through [`collapse_whitespace`]
//! if the user requested it (e.g., via the `--collapse-whitespace` CLI flag).

/// Collapses consecutive empty lines in a string to at most `max_newlines`.
///
/// This function processes the input string line by line, preserving non-empty
/// lines and limiting the number of consecutive blank lines (lines that are
/// empty or contain only whitespace) to the specified maximum.
///
/// # Behavior
///
/// * A line is considered **empty** if, after trimming whitespace characters
///   (spaces, tabs, etc.) with [`str::trim`], it is empty.
/// * Empty lines are collapsed so that no more than `max_newlines` consecutive
///   empty lines appear in the output.
/// * Non-empty lines are always preserved, with exactly one newline between them.
/// * Leading empty lines at the start of the string are subject to the same
///   limit as internal ones.
/// * Trailing newline handling: if the original input ends with a newline,
///   the output will also end with a newline, even if the last line is empty
///   and gets collapsed.
/// * If `max_newlines` is set to [`usize::MAX`], no collapsing is performed
///   and the original string is returned unchanged (except for a possible
///   reallocation).
///
/// # Arguments
///
/// * `input` – The string whose blank lines should be collapsed. It can contain
///   any Unicode text, but only newline characters are considered line separators.
/// * `max_newlines` – The maximum number of consecutive empty lines allowed.
///   Use `0` to remove all empty lines, `1` to allow at most one blank line, etc.
///   Values greater than the actual number of lines have no effect beyond that
///   limit.
///
/// # Returns
///
/// A new `String` with the collapsed content. The output uses Unix newlines (`\n`),
/// regardless of the input's line endings (they are normalised by `.lines()`).
///
/// # Examples
///
/// ## Basic usage
///
/// ```
/// # use comment_remover::core::whitespace::collapse_whitespace;
/// let input = "fn main() {\n    // comment\n\n\n    println!(\"Hello\");\n}\n";
/// // After comment removal we might have multiple blank lines. Collapse to at most 1.
/// let cleaned = collapse_whitespace(input, 1);
/// assert_eq!(
///     cleaned,
///     "fn main() {\n    // comment\n\n    println!(\"Hello\");\n}\n"
/// );
/// ```
///
/// ## Removing all blank lines
///
/// ```
/// # use comment_remover::core::whitespace::collapse_whitespace;
/// let input = "line1\n\n\nline2\n\nline3\n";
/// assert_eq!(collapse_whitespace(input, 0), "line1\nline2\nline3\n");
/// ```
///
/// ## Preserving leading and trailing newlines
///
/// ```
/// # use comment_remover::core::whitespace::collapse_whitespace;
/// let input = "\n\n\ncode();\n\n";
/// // max_newlines = 1
/// assert_eq!(collapse_whitespace(input, 1), "\n\ncode();\n\n");
/// // max_newlines = 0 removes all empty lines, but trailing newline remains.
/// assert_eq!(collapse_whitespace(input, 0), "\ncode();\n");
/// ```
///
/// ## Lines containing spaces are considered empty
///
/// ```
/// # use comment_remover::core::whitespace::collapse_whitespace;
/// let input = "a\n  \n  \nb\n";
/// assert_eq!(collapse_whitespace(input, 1), "a\n  \nb\n");
/// ```
///
/// ## No collapsing
///
/// ```
/// # use comment_remover::core::whitespace::collapse_whitespace;
/// let input = "a\n\nb\n";
/// assert_eq!(collapse_whitespace(input, usize::MAX), input);
/// ```
///
/// # Notes
///
/// * This function does **not** trim leading/trailing whitespace on non-empty lines;
///   only the number of blank lines is controlled.
/// * If you need to also remove trailing spaces from each line, do that as a
///   separate preprocessing step.
/// * The function allocates a new `String`; for very large inputs, consider
///   streaming if performance becomes an issue.
/// Collapses consecutive empty lines in a string to at most `max_newlines`.
///
/// Preserves trailing newline and handles lines with spaces/tabs as empty.
pub fn collapse_whitespace(input: &str, max_newlines: usize) -> String {
    if max_newlines == usize::MAX {
        return input.to_string();
    }

    let lines: Vec<&str> = input.split_inclusive('\n').collect();
    let mut result = String::with_capacity(input.len());
    let mut i = 0;
    let n = lines.len();

    while i < n {
        if lines[i].trim().is_empty() {
            // Awal runtunan baris kosong
            let start = i;
            while i < n && lines[i].trim().is_empty() {
                i += 1;
            }
            let end = i;
            let run_len = end - start;

            let at_start = start == 0;
            let at_end = end == n;

            let keep = if at_start || at_end {
                // Awal/akhir: pertahankan maksimal max_newlines + 1
                if run_len <= max_newlines + 1 {
                    run_len
                } else {
                    max_newlines + 1
                }
            } else {
                // Di dalam: pertahankan maksimal max_newlines
                if run_len <= max_newlines {
                    run_len
                } else {
                    max_newlines
                }
            };

            for j in start..start + keep {
                result.push_str(lines[j]);
            }
        } else {
            // Baris non-kosong
            result.push_str(lines[i]);
            i += 1;
        }
    }

    // Pastikan trailing newline jika input memilikinya
    if input.ends_with('\n') && !result.ends_with('\n') {
        result.push('\n');
    }

    result
}