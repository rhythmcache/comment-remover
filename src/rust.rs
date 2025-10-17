pub fn remove_rust_comments(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // handle byte string literals b"...", br"...", br#"..."#, etc.
        if chars[i] == 'b' && i + 1 < chars.len() {
            if chars[i + 1] == '"' {
                // b"..." byte string literal
                result.push(chars[i]);
                result.push(chars[i + 1]);
                i += 2;
                i = copy_string_contents(&chars, &mut result, i);
                continue;
            } else if chars[i + 1] == '\'' {
                // b'...' byte literal
                result.push(chars[i]);
                result.push(chars[i + 1]);
                i += 2;
                i = copy_char_contents(&chars, &mut result, i);
                continue;
            } else if chars[i + 1] == 'r' {
                // br"..." or br#"..."# raw byte string
                result.push(chars[i]);
                result.push(chars[i + 1]);
                i += 2;
                i = handle_raw_string(&chars, &mut result, i);
                continue;
            }
        }

        // handle raw string literals: r"...", r#"..."#, r##"..."##, etc.
        if chars[i] == 'r' && i + 1 < chars.len() && (chars[i + 1] == '"' || chars[i + 1] == '#') {
            result.push(chars[i]);
            i += 1;
            i = handle_raw_string(&chars, &mut result, i);
            continue;
        }

        // handle single-line comments: // ...
        if i + 1 < chars.len() && chars[i] == '/' && chars[i + 1] == '/' {
            i += 2;
            // skip until end of line
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            // preserve the newline
            if i < chars.len() {
                result.push(chars[i]);
                i += 1;
            }
            continue;
        }

        // handle multi-line comments with nesting /* ... */
        if i + 1 < chars.len() && chars[i] == '/' && chars[i + 1] == '*' {
            i += 2;
            let mut nesting_level = 1;

            while i < chars.len() && nesting_level > 0 {
                // Check for nested block comment start
                if i + 1 < chars.len() && chars[i] == '/' && chars[i + 1] == '*' {
                    nesting_level += 1;
                    i += 2;
                    continue;
                }

                // Check for block comment end
                if i + 1 < chars.len() && chars[i] == '*' && chars[i + 1] == '/' {
                    nesting_level -= 1;
                    i += 2;
                    continue;
                }

                // Preserve newlines to maintain line structure
                if chars[i] == '\n' {
                    result.push('\n');
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

fn handle_raw_string(chars: &[char], result: &mut String, mut i: usize) -> usize {
    // count the number of # before the quote
    let mut hash_count = 0;
    while i < chars.len() && chars[i] == '#' {
        result.push(chars[i]);
        hash_count += 1;
        i += 1;
    }

    // expect opening quote
    if i < chars.len() && chars[i] == '"' {
        result.push(chars[i]);
        i += 1;

        // copy until we find closing quote followed by the matching number of #'s
        while i < chars.len() {
            result.push(chars[i]);

            if chars[i] == '"' {
                // check if this is followed by the right number of #'s
                let mut j = i + 1;
                let mut closing_hashes = 0;
                while j < chars.len() && chars[j] == '#' && closing_hashes < hash_count {
                    closing_hashes += 1;
                    j += 1;
                }

                // if we have the exact number of hashes, it's the end
                if closing_hashes == hash_count {
                    // Copy the closing #'s
                    for _ in 0..hash_count {
                        result.push(chars[i + 1]);
                        i += 1;
                    }
                    i += 1; // move past the last #
                    break;
                }
            }
            i += 1;
        }
    }

    i
}

fn copy_string_contents(chars: &[char], result: &mut String, mut i: usize) -> usize {
    // copy string contents until closing quote, handling escapes
    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() {
            // escape sequence,,, copy both the backslash and next char
            result.push(chars[i]);
            i += 1;
            if i < chars.len() {
                result.push(chars[i]);
                i += 1;
            }
        } else if chars[i] == '"' {
            // Found closing quote
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
            // copy both the backslash and next char
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
        let output = remove_rust_comments(input);
        assert_eq!(output, "int main() {\n    \n    return 0;\n}");
    }

    #[test]
    fn test_multi_line_comment() {
        let input = "int x = 5; /* multi\nline\ncomment */ int y = 10;";
        let output = remove_rust_comments(input);
        assert_eq!(output, "int x = 5; \n\n int y = 10;");
    }

    #[test]
    fn test_nested_block_comments() {
        let input = "int x; /* outer /* inner */ still commented */ int y;";
        let output = remove_rust_comments(input);
        assert_eq!(output, "int x;  int y;");
    }

    #[test]
    fn test_comment_in_string() {
        let input = r#"char* s = "// not a comment";"#;
        let output = remove_rust_comments(input);
        assert_eq!(output, r#"char* s = "// not a comment";"#);
    }

    #[test]
    fn test_comment_in_raw_string() {
        let input = r##"let s = r#"// not a comment /* also not */"#;"##;
        let output = remove_rust_comments(input);
        assert_eq!(output, r##"let s = r#"// not a comment /* also not */"#;"##);
    }

    #[test]
    fn test_raw_string_basic() {
        let input = r#"let s = r"test";"#;
        let output = remove_rust_comments(input);
        assert_eq!(output, r#"let s = r"test";"#);
    }

    #[test]
    fn test_raw_string_with_quotes() {
        let input = r##"let s = r#"test"ing"#;"##;
        let output = remove_rust_comments(input);
        assert_eq!(output, r##"let s = r#"test"ing"#;"##);
    }

    #[test]
    fn test_byte_string() {
        let input = r#"let s = b"// not a comment";"#;
        let output = remove_rust_comments(input);
        assert_eq!(output, r#"let s = b"// not a comment";"#);
    }

    #[test]
    fn test_byte_char() {
        let input = r"let c = b'/'; // actual comment";
        let output = remove_rust_comments(input);
        assert_eq!(output, "let c = b'/'; ");
    }

    #[test]
    fn test_raw_byte_string() {
        let input = r##"let s = br#"/* not a comment */"#;"##;
        let output = remove_rust_comments(input);
        assert_eq!(output, r##"let s = br#"/* not a comment */"#;"##);
    }

    #[test]
    fn test_mixed_comments() {
        let input = "// line comment\nint x; /* block */ int y; // another";
        let output = remove_rust_comments(input);
        assert_eq!(output, "\nint x;  int y; ");
    }

    #[test]
    fn test_escaped_quotes_in_string() {
        let input = r#"char* s = "He said \"Hi // there\"";"#;
        let output = remove_rust_comments(input);
        assert_eq!(output, r#"char* s = "He said \"Hi // there\"";"#);
    }

    #[test]
    fn test_escaped_quotes_in_char() {
        let input = r"let c = '\''; // comment";
        let output = remove_rust_comments(input);
        assert_eq!(output, "let c = '\\''; ");
    }

    #[test]
    fn test_backslash_in_char() {
        let input = r"let c = '\\'; // comment";
        let output = remove_rust_comments(input);
        assert_eq!(output, "let c = '\\\\'; ");
    }

    #[test]
    fn test_complex_nesting() {
        let input = "code /* a /* b /* c */ d */ e */ more";
        let output = remove_rust_comments(input);
        assert_eq!(output, "code  more");
    }

    #[test]
    fn test_string_with_asterisk_slash() {
        let input = r#"let s = "*/"; /* comment */ code"#;
        let output = remove_rust_comments(input);
        assert_eq!(output, r#"let s = "*/";  code"#);
    }

    #[test]
    fn test_multiple_raw_string_hashes() {
        let input = r###"let s = r##"test"#still"##;"###;
        let output = remove_rust_comments(input);
        assert_eq!(output, r###"let s = r##"test"#still"##;"###);
    }
}
