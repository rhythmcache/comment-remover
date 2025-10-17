pub fn remove_go_comments(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // backtick raw strings: `...`
        if chars[i] == '`' {
            result.push(chars[i]);
            i += 1;
            while i < chars.len() {
                result.push(chars[i]);
                if chars[i] == '`' {
                    i += 1;
                    break;
                }
                i += 1;
            }
            continue;
        }

        // double-quoted strings: "..."
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

        // single-quoted runes: '...'
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

        // single-line comments: //
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

        //  block comments: /* */
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

        result.push(chars[i]);
        i += 1;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_line_comments() {
        let input = "x := 5 // this is a comment\ny := 10";
        let expected = "x := 5 \ny := 10";
        assert_eq!(remove_go_comments(input), expected);
    }

    #[test]
    fn test_block_comments() {
        let input = "x := 5 /* this is a comment */ y := 10";
        let expected = "x := 5  y := 10";
        assert_eq!(remove_go_comments(input), expected);
    }

    #[test]
    fn test_multiline_block_comments() {
        let input = "x := 5\n/* this is\n   a multiline\n   comment */\ny := 10";
        let expected = "x := 5\n\n\n\ny := 10";
        assert_eq!(remove_go_comments(input), expected);
    }

    #[test]
    fn test_backtick_raw_strings() {
        let input = "path := `C:\\Users\\test`; // comment";
        let expected = "path := `C:\\Users\\test`; ";
        assert_eq!(remove_go_comments(input), expected);
    }

    #[test]
    fn test_backtick_with_slashes() {
        let input = "regex := `http://example.com`; // comment";
        let expected = "regex := `http://example.com`; ";
        assert_eq!(remove_go_comments(input), expected);
    }

    #[test]
    fn test_double_quoted_strings() {
        let input = "msg := \"hello // world\"; // actual comment";
        let expected = "msg := \"hello // world\"; ";
        assert_eq!(remove_go_comments(input), expected);
    }

    #[test]
    fn test_single_quoted_runes() {
        let input = "ch := 'a'; // character";
        let expected = "ch := 'a'; ";
        assert_eq!(remove_go_comments(input), expected);
    }

    #[test]
    fn test_escaped_strings() {
        let input = "msg := \"hello \\\"world\\\"\"; // comment";
        let expected = "msg := \"hello \\\"world\\\"\"; ";
        assert_eq!(remove_go_comments(input), expected);
    }

    #[test]
    fn test_escaped_runes() {
        let input = "ch := '\\''; // escaped quote";
        let expected = "ch := '\\''; ";
        assert_eq!(remove_go_comments(input), expected);
    }

    #[test]
    fn test_backtick_with_newlines() {
        let input = "msg := `line 1\nline 2 // with slashes\nline 3`; // comment";
        let expected = "msg := `line 1\nline 2 // with slashes\nline 3`; ";
        assert_eq!(remove_go_comments(input), expected);
    }

    #[test]
    fn test_url_in_string() {
        let input = "url := \"http://example.com\"; // this is a comment";
        let expected = "url := \"http://example.com\"; ";
        assert_eq!(remove_go_comments(input), expected);
    }

    #[test]
    fn test_complex_mixed_example() {
        let input = r#"
            // Package comment
            package main
            
            import "fmt" /* standard library */
            
            // Path with slashes
            const Path = `C:\Users\test`
            
            /* Function that does something
               with multiple lines
            */
            func main() {
                msg := "hello // world" // string with comment-like content
                ch := 'a'               /* rune comment */
                path := `http://example.com/path`
                fmt.Println(msg) // print message
            }
        "#;
        let result = remove_go_comments(input);
        
        // check that comments are removed
        assert!(!result.contains("// Package comment"));
        assert!(!result.contains("/* standard library */"));
        assert!(!result.contains("/* Function that does something"));
        
        // check that strings are preserved
        assert!(result.contains(r#""hello // world""#));
        assert!(result.contains("'a'"));
        assert!(result.contains("`C:\\Users\\test`") || result.contains("`C:") || result.contains("`http://example.com/path`"));
    }

    #[test]
    fn test_consecutive_comments() {
        let input = "x := 5 // comment 1\n// comment 2\ny := 10";
        let expected = "x := 5 \n\ny := 10";
        assert_eq!(remove_go_comments(input), expected);
    }

    #[test]
    fn test_empty_string() {
        let input = "";
        let expected = "";
        assert_eq!(remove_go_comments(input), expected);
    }

    #[test]
    fn test_no_comments() {
        let input = "x := 5\ny := 10";
        let expected = "x := 5\ny := 10";
        assert_eq!(remove_go_comments(input), expected);
    }

    #[test]
    fn test_backtick_raw_strings_with_quotes() {
        let input = "msg := `she said \"hello\"`; // comment";
        let expected = "msg := `she said \"hello\"`; ";
        assert_eq!(remove_go_comments(input), expected);
    }

    #[test]
    fn test_division_operator() {
        let input = "result := x / y; // division";
        let expected = "result := x / y; ";
        assert_eq!(remove_go_comments(input), expected);
    }
}
