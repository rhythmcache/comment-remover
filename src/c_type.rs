pub fn remove_c_type_comments(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // handle single-line comments with line continuation: // ... \
        if i + 1 < chars.len() && chars[i] == '/' && chars[i + 1] == '/' {
            i += 2;

            loop {
                // check for line continuation (backslash before newline)
                if i + 1 < chars.len() && chars[i] == '\\' && chars[i + 1] == '\n' {
                    // line continuation,,, skip the backslash and newline, keep going
                    result.push('\n'); // Preserve the newline for line structure
                    i += 2;
                    continue;
                }

                // also handle \r\n line continuation
                if i + 2 < chars.len()
                    && chars[i] == '\\'
                    && chars[i + 1] == '\r'
                    && chars[i + 2] == '\n'
                {
                    result.push('\r');
                    result.push('\n');
                    i += 3;
                    continue;
                }

                // check for end of line without continuation
                if i < chars.len() && chars[i] == '\n' {
                    result.push('\n');
                    i += 1;
                    break;
                }

                // check for \r\n
                if i + 1 < chars.len() && chars[i] == '\r' && chars[i + 1] == '\n' {
                    result.push('\r');
                    result.push('\n');
                    i += 2;
                    break;
                }

                // check for lone \r
                if i < chars.len() && chars[i] == '\r' {
                    result.push('\r');
                    i += 1;
                    break;
                }

                // end of input
                if i >= chars.len() {
                    break;
                }

                // skip comment content
                i += 1;
            }
            continue;
        }

        // handle multi-line comments: /* ... */ (no nesting in C/C++)
        if i + 1 < chars.len() && chars[i] == '/' && chars[i + 1] == '*' {
            i += 2;

            while i < chars.len() {
                // check for block comment end
                if i + 1 < chars.len() && chars[i] == '*' && chars[i + 1] == '/' {
                    i += 2;
                    break;
                }

                // preserve newlines to maintain line structure
                if chars[i] == '\n' {
                    result.push('\n');
                } else if chars[i] == '\r' {
                    result.push('\r');
                }

                i += 1;
            }
            continue;
        }

        // handle regular string literals: "..."
        if chars[i] == '"' {
            result.push(chars[i]);
            i += 1;
            i = copy_string_contents(&chars, &mut result, i);
            continue;
        }

        // handle character literals: '...'
        if chars[i] == '\'' {
            result.push(chars[i]);
            i += 1;
            i = copy_char_contents(&chars, &mut result, i);
            continue;
        }

        // regular character
        result.push(chars[i]);
        i += 1;
    }

    result
}

fn copy_string_contents(chars: &[char], result: &mut String, mut i: usize) -> usize {
    // copy string contents until closing quote, handling escapes and line continuations
    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() {
            // escape sequence copy both the backslash and next char
            result.push(chars[i]);
            i += 1;
            if i < chars.len() {
                result.push(chars[i]);
                i += 1;
            }
        } else if chars[i] == '"' {
            // found closing quote
            result.push(chars[i]);
            i += 1;
            break;
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }
    i
}

fn copy_char_contents(chars: &[char], result: &mut String, mut i: usize) -> usize {
    // copy character literal contents until closing quote, handling escapes
    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() {
            // escape sequence copy both the backslash and next char
            result.push(chars[i]);
            i += 1;
            if i < chars.len() {
                result.push(chars[i]);
                i += 1;
            }
        } else if chars[i] == '\'' {
            // found closing quote
            result.push(chars[i]);
            i += 1;
            break;
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }
    i
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_line_comment() {
        let input = "int main() {\n    // This is a comment\n    return 0;\n}";
        let output = remove_c_type_comments(input);
        assert_eq!(output, "int main() {\n    \n    return 0;\n}");
    }

    #[test]
    fn test_line_continuation_in_comment() {
        let input = "// This comment continues to the next line \\\nint this_should_be_commented = 42;  // But will it be?\nint x = 5;";
        let output = remove_c_type_comments(input);
        assert_eq!(output, "\n\nint x = 5;");
    }

    #[test]
    fn test_multiple_line_continuations() {
        let input = "// Comment \\\ncontinues \\\nand continues \\\nstill going\nint x = 1;";
        let output = remove_c_type_comments(input);
        assert_eq!(output, "\n\n\n\nint x = 1;");
    }

    #[test]
    fn test_line_continuation_crlf() {
        let input = "// Comment \\\r\ncontinues\r\nint x = 1;";
        let output = remove_c_type_comments(input);
        assert_eq!(output, "\r\n\r\nint x = 1;");
    }

    #[test]
    fn test_backslash_not_continuation() {
        let input = "// Comment with \\ backslash but no newline\nint x = 1;";
        let output = remove_c_type_comments(input);
        assert_eq!(output, "\nint x = 1;");
    }

    #[test]
    fn test_multi_line_comment() {
        let input = "int x = 5; /* multi\nline\ncomment */ int y = 10;";
        let output = remove_c_type_comments(input);
        assert_eq!(output, "int x = 5; \n\n int y = 10;");
    }

    #[test]
    fn test_comment_in_string() {
        let input = r#"char* s = "// not a comment";"#;
        let output = remove_c_type_comments(input);
        assert_eq!(output, r#"char* s = "// not a comment";"#);
    }

    #[test]
    fn test_comment_symbols_in_string() {
        let input = r#"char* s = "/* also not a comment */";"#;
        let output = remove_c_type_comments(input);
        assert_eq!(output, r#"char* s = "/* also not a comment */";"#);
    }

    #[test]
    fn test_mixed_comments() {
        let input = "// line comment\nint x; /* block */ int y; // another";
        let output = remove_c_type_comments(input);
        assert_eq!(output, "\nint x;  int y; ");
    }

    #[test]
    fn test_escaped_quotes_in_string() {
        let input = r#"char* s = "He said \"Hi // there\"";"#;
        let output = remove_c_type_comments(input);
        assert_eq!(output, r#"char* s = "He said \"Hi // there\"";"#);
    }

    #[test]
    fn test_escaped_quotes_in_char() {
        let input = r"char c = '\''; // comment";
        let output = remove_c_type_comments(input);
        assert_eq!(output, "char c = '\\''; ");
    }

    #[test]
    fn test_backslash_in_char() {
        let input = r"char c = '\\'; // comment";
        let output = remove_c_type_comments(input);
        assert_eq!(output, "char c = '\\\\'; ");
    }

    #[test]
    fn test_slash_in_char() {
        let input = r"char c = '/'; // actual comment";
        let output = remove_c_type_comments(input);
        assert_eq!(output, "char c = '/'; ");
    }

    #[test]
    fn test_string_with_asterisk_slash() {
        let input = r#"char* s = "*/"; /* comment */ code"#;
        let output = remove_c_type_comments(input);
        assert_eq!(output, r#"char* s = "*/";  code"#);
    }

    #[test]
    fn test_unclosed_block_comment() {
        let input = "int x = 1; /* unclosed comment";
        let output = remove_c_type_comments(input);
        assert_eq!(output, "int x = 1; ");
    }

    #[test]
    fn test_empty_comment() {
        let input = "int x = 1; /**/ int y = 2;";
        let output = remove_c_type_comments(input);
        assert_eq!(output, "int x = 1;  int y = 2;");
    }

    #[test]
    fn test_consecutive_slashes_in_string() {
        let input = r#"char* url = "http://example.com";"#;
        let output = remove_c_type_comments(input);
        assert_eq!(output, r#"char* url = "http://example.com";"#);
    }

    #[test]
    fn test_trigraph_like_sequence() {
        let input = "int x = 1; // Comment??/\nint y = 2;";
        let output = remove_c_type_comments(input);
        // this doesn't handle trigraphs
        assert_eq!(output, "int x = 1; \nint y = 2;");
    }

    #[test]
    fn test_line_continuation_at_eof() {
        let input = "// Comment with continuation \\";
        let output = remove_c_type_comments(input);
        assert_eq!(output, "");
    }

    #[test]
    fn test_complex_example() {
        let input = r#"
int main() {
    // This is a comment \
    that continues here
    char* s = "// not a comment";
    int x = 5; /* block
    comment */ int y = 10;
    char c = '/'; // actual comment
    return 0;
}
"#;
        let expected = r#"
int main() {
    

    char* s = "// not a comment";
    int x = 5; 
 int y = 10;
    char c = '/'; 
    return 0;
}
"#;
        let output = remove_c_type_comments(input);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_string_with_newline_escape() {
        let input = r#"char* s = "line1\nline2"; // comment"#;
        let output = remove_c_type_comments(input);
        assert_eq!(output, r#"char* s = "line1\nline2"; "#);
    }

    #[test]
    fn test_multiline_string_with_backslash() {
        let input = "char* s = \"line1\\\nline2\"; // comment";
        let output = remove_c_type_comments(input);
        assert_eq!(output, "char* s = \"line1\\\nline2\"; ");
    }
}
