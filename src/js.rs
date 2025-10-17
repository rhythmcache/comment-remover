pub fn remove_js_comments(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // handle strings: "..."
        if chars[i] == '"' {
            result.push(chars[i]);
            i += 1;
            while i < chars.len() {
                result.push(chars[i]);
                if chars[i] == '\\' && i + 1 < chars.len() {
                    i += 1;
                    result.push(chars[i]);
                    i += 1;
                } else if chars[i] == '"' {
                    i += 1;
                    break;
                } else {
                    i += 1;
                }
            }
            continue;
        }

        // handle strings: '...'
        if chars[i] == '\'' {
            result.push(chars[i]);
            i += 1;
            while i < chars.len() {
                result.push(chars[i]);
                if chars[i] == '\\' && i + 1 < chars.len() {
                    i += 1;
                    result.push(chars[i]);
                    i += 1;
                } else if chars[i] == '\'' {
                    i += 1;
                    break;
                } else {
                    i += 1;
                }
            }
            continue;
        }

        // handle template literals: `...${...}...`
        if chars[i] == '`' {
            result.push(chars[i]);
            i += 1;
            while i < chars.len() {
                if chars[i] == '\\' && i + 1 < chars.len() {
                    result.push(chars[i]);
                    i += 1;
                    result.push(chars[i]);
                    i += 1;
                } else if chars[i] == '`' {
                    result.push(chars[i]);
                    i += 1;
                    break;
                } else if chars[i] == '$' && i + 1 < chars.len() && chars[i + 1] == '{' {
                    // handle ${...} expressions inside template literals
                    result.push(chars[i]); // $
                    i += 1;
                    result.push(chars[i]); // {
                    i += 1;

                    // track brace depth to find the closing }
                    let mut brace_depth = 1;
                    while i < chars.len() && brace_depth > 0 {
                        if chars[i] == '{' {
                            brace_depth += 1;
                        } else if chars[i] == '}' {
                            brace_depth -= 1;
                        }
                        // recursively process the expression inside ${}
                        // by checking for strings and comments within
                        result.push(chars[i]);
                        i += 1;
                    }
                } else {
                    result.push(chars[i]);
                    i += 1;
                }
            }
            continue;
        }

        // handle single-line comments: //
        if i + 1 < chars.len() && chars[i] == '/' && chars[i + 1] == '/' {
            i += 2;
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            if i < chars.len() {
                result.push(chars[i]); // keep the newline
                i += 1;
            }
            continue;
        }

        // handle block comments: /* */
        if i + 1 < chars.len() && chars[i] == '/' && chars[i + 1] == '*' {
            i += 2;
            while i + 1 < chars.len() {
                if chars[i] == '*' && chars[i + 1] == '/' {
                    i += 2;
                    break;
                }
                if chars[i] == '\n' {
                    result.push('\n'); // preserve newlines for line counts
                }
                i += 1;
            }
            continue;
        }

        // handle regex literals: /pattern/flags
        // this is trickycz we need to detect if / starts a regex or is division
        // simple heuristic: after these tokens, / is likely a regex
        if chars[i] == '/' && is_regex_context(&result) {
            result.push(chars[i]);
            i += 1;
            while i < chars.len() {
                result.push(chars[i]);
                if chars[i] == '\\' && i + 1 < chars.len() {
                    i += 1;
                    result.push(chars[i]);
                    i += 1;
                } else if chars[i] == '/' {
                    i += 1;
                    // consume flags (g, i, m, s, u, y)
                    while i < chars.len() && matches!(chars[i], 'g' | 'i' | 'm' | 's' | 'u' | 'y') {
                        result.push(chars[i]);
                        i += 1;
                    }
                    break;
                } else {
                    i += 1;
                }
            }
            continue;
        }

        result.push(chars[i]);
        i += 1;
    }

    result
}

// heuristic to determine if / starts a regex or is division
// returns true if / likely starts a regex based on preceding context
fn is_regex_context(result: &str) -> bool {
    let trimmed = result.trim_end();
    if trimmed.is_empty() {
        return true;
    }

    // get the last non-whitespace character
    let last_char = trimmed.chars().last().unwrap();

    // / is definitely division after closing brackets/parens or identifiers/numbers
    if matches!(last_char, ')' | ']' | '}' | '0'..='9') {
        return false;
    }

    // / is definitely division after string/template endings (though shouldn't reach here)
    if matches!(last_char, '"' | '\'' | '`') {
        return false;
    }

    // / is definitely regex after operators and control structures
    if matches!(
        last_char,
        '=' | '('
            | '['
            | ','
            | ':'
            | ';'
            | '!'
            | '&'
            | '|'
            | '?'
            | '+'
            | '-'
            | '*'
            | '%'
            | '^'
            | '~'
            | '{'
    ) {
        return true;
    }

    // Check for keywords that should be followed by regex
    // Use word boundary checking to avoid false positives like "returnValue"
    if is_keyword_prefix(trimmed, "return") {
        return true;
    }
    if is_keyword_prefix(trimmed, "throw") {
        return true;
    }
    if is_keyword_prefix(trimmed, "new") {
        return true;
    }
    if is_keyword_prefix(trimmed, "case") {
        return true;
    }
    if is_keyword_prefix(trimmed, "delete") {
        return true;
    }
    if is_keyword_prefix(trimmed, "void") {
        return true;
    }
    if is_keyword_prefix(trimmed, "typeof") {
        return true;
    }
    if is_keyword_prefix(trimmed, "instanceof") {
        return true;
    }
    if is_keyword_prefix(trimmed, "in") {
        return true;
    }

    // default behaviour is to assume division (safer default)
    false
}

// check if trimmed string ends with a keyword followed by only whitespace
// ensures "returnValue / 2" doesn't match "return"
fn is_keyword_prefix(trimmed: &str, keyword: &str) -> bool {
    if !trimmed.ends_with(keyword) {
        return false;
    }

    // if keyword is the entire string, it matches
    if trimmed.len() == keyword.len() {
        return true;
    }

    // check character before keyword - must not be alphanumeric or underscore
    let char_before_idx = trimmed.len() - keyword.len() - 1;
    if char_before_idx < trimmed.len() {
        let before_char = trimmed.chars().nth(char_before_idx);
        if let Some(c) = before_char {
            return !c.is_alphanumeric() && c != '_';
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_line_comments() {
        let input = "let x = 5; // this is a comment\nlet y = 10;";
        let expected = "let x = 5; \nlet y = 10;";
        assert_eq!(remove_js_comments(input), expected);
    }

    #[test]
    fn test_block_comments() {
        let input = "let x = 5; /* this is a comment */ let y = 10;";
        let expected = "let x = 5;  let y = 10;";
        assert_eq!(remove_js_comments(input), expected);
    }

    #[test]
    fn test_multiline_block_comments() {
        let input = "let x = 5;\n/* this is\n   a multiline\n   comment */\nlet y = 10;";
        let expected = "let x = 5;\n\n\n\nlet y = 10;";
        assert_eq!(remove_js_comments(input), expected);
    }

    #[test]
    fn test_double_quoted_strings() {
        let input = r#"let msg = "hello // world"; // actual comment"#;
        let expected = r#"let msg = "hello // world"; "#;
        assert_eq!(remove_js_comments(input), expected);
    }

    #[test]
    fn test_single_quoted_strings() {
        let input = "let msg = 'hello // world'; // actual comment";
        let expected = "let msg = 'hello // world'; ";
        assert_eq!(remove_js_comments(input), expected);
    }

    #[test]
    fn test_template_literals() {
        let input = "let msg = `hello // world`; // actual comment";
        let expected = "let msg = `hello // world`; ";
        assert_eq!(remove_js_comments(input), expected);
    }

    #[test]
    fn test_template_literals_with_expressions() {
        let input = "let msg = `value: ${x /* inner comment */ + 5}`; // outer comment";
        let expected = "let msg = `value: ${x /* inner comment */ + 5}`; ";
        assert_eq!(remove_js_comments(input), expected);
    }

    #[test]
    fn test_template_literals_with_nested_braces() {
        let input = "let msg = `result: ${obj.method({a: 1})}`;";
        let expected = "let msg = `result: ${obj.method({a: 1})}`;";
        assert_eq!(remove_js_comments(input), expected);
    }

    #[test]
    fn test_escaped_strings() {
        let input = r#"let msg = "hello \"world\""; // comment"#;
        let expected = r#"let msg = "hello \"world\""; "#;
        assert_eq!(remove_js_comments(input), expected);
    }

    #[test]
    fn test_regex_literal_after_equals() {
        let input = "let pattern = /test/gi; // comment";
        let expected = "let pattern = /test/gi; ";
        assert_eq!(remove_js_comments(input), expected);
    }

    #[test]
    fn test_regex_literal_after_return() {
        let input = "return /test/g; // comment";
        let expected = "return /test/g; ";
        assert_eq!(remove_js_comments(input), expected);
    }

    #[test]
    fn test_regex_literal_with_escaped_slashes() {
        let input = r"let pattern = /test\/path/; // comment";
        let expected = r"let pattern = /test\/path/; ";
        assert_eq!(remove_js_comments(input), expected);
    }

    #[test]
    fn test_division_not_treated_as_regex() {
        let input = "let result = x / 2; // comment";
        let expected = "let result = x / 2; ";
        assert_eq!(remove_js_comments(input), expected);
    }

    #[test]
    fn test_complex_mixed_example() {
        let input = r#"
            // Initial comment
            const pattern = /test\/\d+/gi; // regex with flags
            const str = "hello // world"; /* block comment */
            const tpl = `value: ${x + 5}`; // template literal
            let result = x / 2; // division, not regex
            /* multiline
               comment here */
            function test() {
                return /^test$/; // regex after return
            }
        "#;
        let result = remove_js_comments(input);

        // Check that comments are removed
        assert!(!result.contains("// Initial comment"));
        assert!(!result.contains("/* block comment */"));
        assert!(!result.contains("/* multiline"));

        // Check that strings and patterns are preserved
        assert!(result.contains(r#""hello // world""#));
        assert!(result.contains("/test\\/\\d+/gi"));
        assert!(result.contains("`value: ${x + 5}`"));
        assert!(result.contains("return /^test$/"));
    }

    #[test]
    fn test_comment_with_slashes_in_string() {
        let input = r#"let url = "http://example.com"; // this is a comment"#;
        let expected = r#"let url = "http://example.com"; "#;
        assert_eq!(remove_js_comments(input), expected);
    }

    #[test]
    fn test_empty_string() {
        let input = "";
        let expected = "";
        assert_eq!(remove_js_comments(input), expected);
    }

    #[test]
    fn test_only_comments() {
        let input = "// comment 1\n/* comment 2 */";
        let expected = "\n";
        assert_eq!(remove_js_comments(input), expected);
    }

    #[test]
    fn test_no_comments() {
        let input = "let x = 5;\nlet y = 10;";
        let expected = "let x = 5;\nlet y = 10;";
        assert_eq!(remove_js_comments(input), expected);
    }

    #[test]
    fn test_consecutive_comments() {
        let input = "x; // comment 1\n// comment 2\ny;";
        let expected = "x; \n\ny;";
        assert_eq!(remove_js_comments(input), expected);
    }
}
