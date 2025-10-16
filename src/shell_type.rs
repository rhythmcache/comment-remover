/// helper to extract and preserve shebang if present
/// returns (shebang_with_newline, remaining_input)
fn extract_shebang(input: &str) -> (String, &str) {
    if input.starts_with("#!") {
        if let Some(newline_pos) = input.find('\n') {
            let shebang = input[..=newline_pos].to_string();
            let rest = &input[newline_pos + 1..];
            return (shebang, rest);
        } else {
            // shebang is the entire file
            return (input.to_string(), "");
        }
    }
    (String::new(), input)
}

/// basic # comment remover handles simple cases
/// works for most languages with # comments when they don't have special syntax
pub fn remove_hash_comments_basic(input: &str) -> String {
    let (shebang, content) = extract_shebang(input);

    let mut result = String::with_capacity(input.len());
    let chars: Vec<char> = content.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '#' {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            if i < chars.len() {
                result.push(chars[i]);
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

    shebang + &result
}

/// remove shell-style comments with heredoc support
/// use this for Bash, Zsh, and other shell scripts
pub fn remove_shell_comments(input: &str) -> String {
    let (shebang, content) = extract_shebang(input);

    let mut result = String::with_capacity(input.len());
    let chars: Vec<char> = content.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // try to handle heredoc
        if let Some(new_pos) = handle_heredoc(&chars, i, &mut result) {
            i = new_pos;
            continue;
        }

        // check for escaped character (including \#)
        if chars[i] == '\\' && i + 1 < chars.len() {
            if chars[i + 1] == '\n' {
                // line continuation
                result.push(chars[i]);
                result.push(chars[i + 1]);
                i += 2;

                // skip whitespace on next line
                while i < chars.len() && (chars[i] == ' ' || chars[i] == '\t') {
                    result.push(chars[i]);
                    i += 1;
                }
            } else {
                // escaped character (like \#, \$, etc.)
                result.push(chars[i]);
                result.push(chars[i + 1]);
                i += 2;
            }
            continue;
        }

        // check for # comment (only if preceded by whitespace or at line start)
        if chars[i] == '#' {
            // check if this looks like a real comment start
            let is_comment = i == 0 || chars[i - 1].is_whitespace();

            if is_comment {
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
                if i < chars.len() {
                    result.push(chars[i]);
                    i += 1;
                }
                continue;
            }
        }

        // check for double-quoted strings
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

        // check for single-quoted strings
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

        // regular character
        result.push(chars[i]);
        i += 1;
    }

    shebang + &result
}

/// remove python comments (# and triple-quoted strings used as comments)
/// handles both """ and ''' docstrings/comments
pub fn remove_python_comments(input: &str) -> String {
    let (shebang, content) = extract_shebang(input);

    let mut result = String::with_capacity(input.len());
    let chars: Vec<char> = content.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // check for triple-quoted strings (""" or ''')
        if i + 2 < chars.len() {
            let is_triple_double = chars[i] == '"' && chars[i + 1] == '"' && chars[i + 2] == '"';
            let is_triple_single = chars[i] == '\'' && chars[i + 1] == '\'' && chars[i + 2] == '\'';

            if is_triple_double || is_triple_single {
                let quote_char = chars[i];

                // copy the opening quotes
                result.push(chars[i]);
                result.push(chars[i + 1]);
                result.push(chars[i + 2]);
                i += 3;

                // find the closing triple quotes
                while i + 2 < chars.len() {
                    if chars[i] == quote_char
                        && chars[i + 1] == quote_char
                        && chars[i + 2] == quote_char
                    {
                        // found closing quotes
                        result.push(chars[i]);
                        result.push(chars[i + 1]);
                        result.push(chars[i + 2]);
                        i += 3;
                        break;
                    } else {
                        result.push(chars[i]);
                        i += 1;
                    }
                }
                continue;
            }
        }

        // check for # comment
        if chars[i] == '#' {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            if i < chars.len() {
                result.push(chars[i]);
                i += 1;
            }
            continue;
        }

        // check for regular double-quoted strings
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

        // check for regular single-quoted strings
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

        // regular character
        result.push(chars[i]);
        i += 1;
    }

    shebang + &result
}

fn handle_heredoc(chars: &[char], mut i: usize, result: &mut String) -> Option<usize> {
    // check for heredoc(<<EOF, <<'EOF', <<"EOF", <<-EOF)
    if i + 2 >= chars.len() || chars[i] != '<' || chars[i + 1] != '<' {
        return None;
    }

    // copy the << part
    result.push(chars[i]);
    result.push(chars[i + 1]);
    i += 2;

    // skip optional dash for <<-
    if i < chars.len() && chars[i] == '-' {
        result.push(chars[i]);
        i += 1;
    }

    // skip whitespace
    while i < chars.len() && (chars[i] == ' ' || chars[i] == '\t') {
        result.push(chars[i]);
        i += 1;
    }

    // check if delimiter is quoted
    let mut quoted = false;
    let quote_char = if i < chars.len() && (chars[i] == '\'' || chars[i] == '"') {
        quoted = true;
        let q = chars[i];
        result.push(chars[i]);
        i += 1;
        q
    } else {
        '\0'
    };

    // extract delimiter
    let mut delimiter = String::new();
    while i < chars.len() && chars[i] != '\n' {
        if quoted && chars[i] == quote_char {
            result.push(chars[i]);
            i += 1;
            break;
        } else if !quoted
            && (chars[i] == ' ' || chars[i] == '\t' || chars[i] == ';' || chars[i] == '&')
        {
            break;
        } else {
            delimiter.push(chars[i]);
            result.push(chars[i]);
            i += 1;
        }
    }

    // copy rest of line
    while i < chars.len() && chars[i] != '\n' {
        result.push(chars[i]);
        i += 1;
    }
    if i < chars.len() {
        result.push(chars[i]); // newline
        i += 1;
    }

    // copy everything until we hit the delimiter
    if !delimiter.is_empty() {
        while i < chars.len() {
            let mut line = String::new();

            // read the line
            while i < chars.len() && chars[i] != '\n' {
                line.push(chars[i]);
                i += 1;
            }

            // check if this line is the delimiter
            if line.trim() == delimiter {
                // Copy the delimiter line and we're done
                for c in line.chars() {
                    result.push(c);
                }
                if i < chars.len() {
                    result.push(chars[i]); // newline
                    i += 1;
                }
                break;
            } else {
                // not the delimiter, copy the line as-is
                for c in line.chars() {
                    result.push(c);
                }
                if i < chars.len() {
                    result.push(chars[i]); // newline
                    i += 1;
                }
            }
        }
    }

    Some(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_hash_comment() {
        let input = "x = 5  # inline comment\ny = 10";
        let output = remove_hash_comments_basic(input);
        assert_eq!(output, "x = 5  \ny = 10");
    }

    #[test]
    fn test_basic_with_strings() {
        let input = "echo \"# not a comment\"";
        let output = remove_hash_comments_basic(input);
        assert_eq!(output, "echo \"# not a comment\"");
    }

    #[test]
    fn test_basic_with_shebang() {
        let input = "#!/bin/bash\n# This is a comment\necho hello";
        let output = remove_hash_comments_basic(input);
        assert_eq!(output, "#!/bin/bash\n\necho hello");
    }

    #[test]
    fn test_shell_heredoc_unquoted() {
        let input = "cat <<EOF\n# This is not a comment\nSome text\nEOF\necho done";
        let output = remove_shell_comments(input);
        assert_eq!(
            output,
            "cat <<EOF\n# This is not a comment\nSome text\nEOF\necho done"
        );
    }

    #[test]
    fn test_shell_heredoc_quoted() {
        let input = "cat <<'EOF'\n# This is not a comment\nSome text\nEOF\necho done";
        let output = remove_shell_comments(input);
        assert_eq!(
            output,
            "cat <<'EOF'\n# This is not a comment\nSome text\nEOF\necho done"
        );
    }

    #[test]
    fn test_shell_line_continuation() {
        let input = "echo \"Hello\" \\\n# This is a comment\nWorld";
        let output = remove_shell_comments(input);
        assert_eq!(output, "echo \"Hello\" \\\n\nWorld");
    }

    #[test]
    fn test_shell_with_shebang() {
        let input = "#!/usr/bin/env python3\n# This is a comment\nprint('hello')";
        let output = remove_shell_comments(input);
        assert_eq!(output, "#!/usr/bin/env python3\n\nprint('hello')");
    }

    #[test]
    fn test_shell_escaped_hash() {
        let input = "echo \\#escaped # comment here";
        let output = remove_shell_comments(input);
        assert_eq!(output, "echo \\#escaped ");
    }

    #[test]
    fn test_shell_hash_in_token() {
        let input = "VAR=value#notacomment\necho $VAR # real comment";
        let output = remove_shell_comments(input);
        assert_eq!(output, "VAR=value#notacomment\necho $VAR ");
    }

    #[test]
    fn test_shell_line_continuation_with_comment() {
        let input = "echo \"Hello\" \\  # line continues\n&& echo \"World\"";
        let output = remove_shell_comments(input);
        assert_eq!(output, "echo \"Hello\" \\  \n&& echo \"World\"");
    }

    #[test]
    fn test_python_triple_quotes_double() {
        let input = "def foo():\n    \"\"\"This is a docstring\n    # Not a comment\n    \"\"\"\n    x = 1  # real comment";
        let output = remove_python_comments(input);
        assert_eq!(
            output,
            "def foo():\n    \"\"\"This is a docstring\n    # Not a comment\n    \"\"\"\n    x = 1  "
        );
    }

    #[test]
    fn test_python_triple_quotes_single() {
        let input = "x = '''multi\nline\n# string'''\ny = 5  # comment";
        let output = remove_python_comments(input);
        assert_eq!(output, "x = '''multi\nline\n# string'''\ny = 5  ");
    }

    #[test]
    fn test_python_regular_comment() {
        let input = "# Python comment\ndef foo():\n    # indented comment\n    x = \"#notcomment\"\n    return x  # end comment";
        let output = remove_python_comments(input);
        assert_eq!(
            output,
            "\ndef foo():\n    \n    x = \"#notcomment\"\n    return x  "
        );
    }

    #[test]
    fn test_python_with_shebang() {
        let input = "#!/usr/bin/python3\n# Comment\ndef foo(): pass";
        let output = remove_python_comments(input);
        assert_eq!(output, "#!/usr/bin/python3\n\ndef foo(): pass");
    }
}
