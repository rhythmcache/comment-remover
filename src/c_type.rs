pub fn remove_c_type_comments(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if i + 1 < chars.len() && chars[i] == '/' && chars[i + 1] == '/' {
            i += 2;
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            if i < chars.len() {
                result.push(chars[i]);
                i += 1;
            }
            continue;
        }
        if i + 1 < chars.len() && chars[i] == '/' && chars[i + 1] == '*' {
            i += 2;
            while i + 1 < chars.len() {
                if chars[i] == '*' && chars[i + 1] == '/' {
                    i += 2;
                    break;
                }
                if chars[i] == '\n' {
                    result.push('\n');
                }
                i += 1;
            }
            continue;
        }
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
        result.push(chars[i]);
        i += 1;
    }

    result
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
    fn test_mixed_comments() {
        let input = "// line comment\nint x; /* block */ int y; // another";
        let output = remove_c_type_comments(input);
        assert_eq!(output, "\nint x;  int y; ");
    }

    #[test]
    fn test_escaped_quotes() {
        let input = r#"char* s = "He said \"Hi // there\"";"#;
        let output = remove_c_type_comments(input);
        assert_eq!(output, r#"char* s = "He said \"Hi // there\"";"#);
    }
}
