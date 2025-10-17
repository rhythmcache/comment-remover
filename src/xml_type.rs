pub fn xml_type_remover(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if i + 3 < chars.len()
            && chars[i] == '<'
            && chars[i + 1] == '!'
            && chars[i + 2] == '-'
            && chars[i + 3] == '-'
        {
            i += 4;
            let mut newline_count = 0;

            while i < chars.len() {
                if i + 2 < chars.len()
                    && chars[i] == '-'
                    && chars[i + 1] == '-'
                    && chars[i + 2] == '>'
                {
                    i += 3;
                    break;
                }

                if chars[i] == '\n' {
                    newline_count += 1;
                }
                i += 1;
            }

            for _ in 0..newline_count {
                result.push('\n');
            }
            continue;
        }

        if i + 8 < chars.len()
            && chars[i] == '<'
            && chars[i + 1] == '!'
            && chars[i + 2] == '['
            && chars[i + 3] == 'C'
            && chars[i + 4] == 'D'
            && chars[i + 5] == 'A'
            && chars[i + 6] == 'T'
            && chars[i + 7] == 'A'
            && chars[i + 8] == '['
        {
            result.push_str("<![CDATA[");
            i += 9;

            while i < chars.len() {
                if i + 2 < chars.len()
                    && chars[i] == ']'
                    && chars[i + 1] == ']'
                    && chars[i + 2] == '>'
                {
                    result.push_str("]]>");
                    i += 3;
                    break;
                }
                result.push(chars[i]);
                i += 1;
            }
            continue;
        }

        if i + 7 < chars.len()
            && chars[i] == '<'
            && (chars[i + 1] == 's' || chars[i + 1] == 'S')
            && (chars[i + 2] == 'c' || chars[i + 2] == 'C')
            && (chars[i + 3] == 'r' || chars[i + 3] == 'R')
            && (chars[i + 4] == 'i' || chars[i + 4] == 'I')
            && (chars[i + 5] == 'p' || chars[i + 5] == 'P')
            && (chars[i + 6] == 't' || chars[i + 6] == 'T')
            && (chars[i + 7] == '>' || chars[i + 7] == ' ')
        {
            while i < chars.len() && chars[i] != '>' {
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
            if i < chars.len() {
                result.push(chars[i]);
                i += 1;
            }

            while i < chars.len() {
                if i + 8 < chars.len()
                    && chars[i] == '<'
                    && chars[i + 1] == '/'
                    && (chars[i + 2] == 's' || chars[i + 2] == 'S')
                    && (chars[i + 3] == 'c' || chars[i + 3] == 'C')
                    && (chars[i + 4] == 'r' || chars[i + 4] == 'R')
                    && (chars[i + 5] == 'i' || chars[i + 5] == 'I')
                    && (chars[i + 6] == 'p' || chars[i + 6] == 'P')
                    && (chars[i + 7] == 't' || chars[i + 7] == 'T')
                    && chars[i + 8] == '>'
                {
                    result.push_str("</script>");
                    i += 9;

                    break;
                }

                if i + 8 < chars.len()
                    && chars[i] == '<'
                    && chars[i + 1] == '!'
                    && chars[i + 2] == '['
                    && chars[i + 3] == 'C'
                    && chars[i + 4] == 'D'
                    && chars[i + 5] == 'A'
                    && chars[i + 6] == 'T'
                    && chars[i + 7] == 'A'
                    && chars[i + 8] == '['
                {
                    result.push_str("<![CDATA[");
                    i += 9;

                    while i < chars.len() {
                        if i + 2 < chars.len()
                            && chars[i] == ']'
                            && chars[i + 1] == ']'
                            && chars[i + 2] == '>'
                        {
                            result.push_str("]]>");
                            i += 3;
                            break;
                        }
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

                if i + 3 < chars.len()
                    && chars[i] == '<'
                    && chars[i + 1] == '!'
                    && chars[i + 2] == '-'
                    && chars[i + 3] == '-'
                {
                    i += 4;
                    let mut newline_count = 0;
                    while i < chars.len() {
                        if i + 2 < chars.len()
                            && chars[i] == '-'
                            && chars[i + 1] == '-'
                            && chars[i + 2] == '>'
                        {
                            i += 3;
                            break;
                        }
                        if chars[i] == '\n' {
                            newline_count += 1;
                        }
                        i += 1;
                    }
                    for _ in 0..newline_count {
                        result.push('\n');
                    }
                    continue;
                }

                result.push(chars[i]);
                i += 1;
            }
            continue;
        }

        if i + 6 < chars.len()
            && chars[i] == '<'
            && (chars[i + 1] == 's' || chars[i + 1] == 'S')
            && (chars[i + 2] == 't' || chars[i + 2] == 'T')
            && (chars[i + 3] == 'y' || chars[i + 3] == 'Y')
            && (chars[i + 4] == 'l' || chars[i + 4] == 'L')
            && (chars[i + 5] == 'e' || chars[i + 5] == 'E')
            && (chars[i + 6] == '>' || chars[i + 6] == ' ')
        {
            while i < chars.len() && chars[i] != '>' {
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
            if i < chars.len() {
                result.push(chars[i]);
                i += 1;
            }

            while i < chars.len() {
                if i + 7 < chars.len()
                    && chars[i] == '<'
                    && chars[i + 1] == '/'
                    && (chars[i + 2] == 's' || chars[i + 2] == 'S')
                    && (chars[i + 3] == 't' || chars[i + 3] == 'T')
                    && (chars[i + 4] == 'y' || chars[i + 4] == 'Y')
                    && (chars[i + 5] == 'l' || chars[i + 5] == 'L')
                    && (chars[i + 6] == 'e' || chars[i + 6] == 'E')
                    && chars[i + 7] == '>'
                {
                    result.push_str("</style>");
                    i += 8;

                    break;
                }

                if i + 8 < chars.len()
                    && chars[i] == '<'
                    && chars[i + 1] == '!'
                    && chars[i + 2] == '['
                    && chars[i + 3] == 'C'
                    && chars[i + 4] == 'D'
                    && chars[i + 5] == 'A'
                    && chars[i + 6] == 'T'
                    && chars[i + 7] == 'A'
                    && chars[i + 8] == '['
                {
                    result.push_str("<![CDATA[");
                    i += 9;

                    while i < chars.len() {
                        if i + 2 < chars.len()
                            && chars[i] == ']'
                            && chars[i + 1] == ']'
                            && chars[i + 2] == '>'
                        {
                            result.push_str("]]>");
                            i += 3;
                            break;
                        }
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

                if i + 3 < chars.len()
                    && chars[i] == '<'
                    && chars[i + 1] == '!'
                    && chars[i + 2] == '-'
                    && chars[i + 3] == '-'
                {
                    i += 4;
                    let mut newline_count = 0;
                    while i < chars.len() {
                        if i + 2 < chars.len()
                            && chars[i] == '-'
                            && chars[i + 1] == '-'
                            && chars[i + 2] == '>'
                        {
                            i += 3;
                            break;
                        }
                        if chars[i] == '\n' {
                            newline_count += 1;
                        }
                        i += 1;
                    }
                    for _ in 0..newline_count {
                        result.push('\n');
                    }
                    continue;
                }

                result.push(chars[i]);
                i += 1;
            }
            continue;
        }

        if chars[i] == '<' && i + 1 < chars.len() && chars[i + 1] != '!' && chars[i + 1] != '?' {
            result.push(chars[i]);
            i += 1;

            loop {
                if i >= chars.len() {
                    break;
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

                if chars[i] == '>' {
                    result.push(chars[i]);
                    i += 1;
                    break;
                }

                result.push(chars[i]);
                i += 1;
            }
            continue;
        }

        if i + 1 < chars.len() && chars[i] == '<' && chars[i + 1] == '?' {
            result.push(chars[i]);
            i += 1;
            result.push(chars[i]);
            i += 1;
            while i + 1 < chars.len() {
                result.push(chars[i]);
                if chars[i] == '?' && chars[i + 1] == '>' {
                    i += 1;
                    result.push(chars[i]);
                    i += 1;
                    break;
                }
                i += 1;
            }
            continue;
        }

        if i + 1 < chars.len() && chars[i] == '<' && chars[i + 1] == '!' {
            result.push(chars[i]);
            i += 1;
            while i < chars.len() && chars[i] != '>' {
                result.push(chars[i]);
                i += 1;
            }
            if i < chars.len() {
                result.push(chars[i]);
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
    fn test_simple_comment() {
        let input = r#"<root><!-- This is a comment --><element /></root>"#;
        let expected = "<root><element /></root>";
        assert_eq!(xml_type_remover(input), expected);
    }

    #[test]
    fn test_multiple_comments() {
        let input =
            r#"<!-- Comment 1 --><root><!-- Comment 2 --><child /></root><!-- Comment 3 -->"#;
        let expected = "<root><child /></root>";
        assert_eq!(xml_type_remover(input), expected);
    }

    #[test]
    fn test_comment_with_newlines() {
        let input = "<!-- Comment\nwith\nnewlines --><root></root>";
        let expected = "\n\n<root></root>";
        assert_eq!(xml_type_remover(input), expected);
    }

    #[test]
    fn test_cdata_preserved() {
        let input = r#"<root><![CDATA[Some <content> with <!-- fake comment -->]]></root>"#;
        let expected = r#"<root><![CDATA[Some <content> with <!-- fake comment -->]]></root>"#;
        assert_eq!(xml_type_remover(input), expected);
    }

    #[test]
    fn test_cdata_with_newlines() {
        let input = "<root><![CDATA[Line 1\nLine 2\nLine 3]]></root>";
        let expected = "<root><![CDATA[Line 1\nLine 2\nLine 3]]></root>";
        assert_eq!(xml_type_remover(input), expected);
    }

    #[test]
    fn test_string_with_comment_markers_double_quotes() {
        let input = r#"<root attr="<!-- not a comment -->"><child></child></root>"#;
        let expected = r#"<root attr="<!-- not a comment -->"><child></child></root>"#;
        assert_eq!(xml_type_remover(input), expected);
    }

    #[test]
    fn test_single_quote_string() {
        let input = r#"<root attr='<!-- not a comment -->'><child /></root>"#;
        let expected = r#"<root attr='<!-- not a comment -->'><child /></root>"#;
        assert_eq!(xml_type_remover(input), expected);
    }

    #[test]
    fn test_escaped_quotes_in_string() {
        let input = r#"<root attr="value with \" escaped <!-- fake --> quote"><child /></root>"#;
        let expected = r#"<root attr="value with \" escaped <!-- fake --> quote"><child /></root>"#;
        assert_eq!(xml_type_remover(input), expected);
    }

    #[test]
    fn test_comment_with_dashes() {
        let input = r#"<!-- Comment - with - dashes --><root />"#;
        let expected = "<root />";
        assert_eq!(xml_type_remover(input), expected);
    }

    #[test]
    fn test_unclosed_comment() {
        let input = r#"<root><!-- Unclosed comment"#;
        let expected = "<root>";
        assert_eq!(xml_type_remover(input), expected);
    }

    #[test]
    fn test_unclosed_cdata() {
        let input = r#"<root><![CDATA[Unclosed CDATA"#;
        let expected = "<root><![CDATA[Unclosed CDATA";
        assert_eq!(xml_type_remover(input), expected);
    }

    #[test]
    fn test_mixed_comments_and_cdata() {
        let input = r#"<!-- Comment --><root><![CDATA[<!-- preserved -->]]><!-- Another comment --></root>"#;
        let expected = r#"<root><![CDATA[<!-- preserved -->]]></root>"#;
        assert_eq!(xml_type_remover(input), expected);
    }

    #[test]
    fn test_empty_comment() {
        let input = "<!----><root></root>";
        let expected = "<root></root>";
        assert_eq!(xml_type_remover(input), expected);
    }

    #[test]
    fn test_empty_cdata() {
        let input = "<root><![CDATA[]]></root>";
        let expected = "<root><![CDATA[]]></root>";
        assert_eq!(xml_type_remover(input), expected);
    }

    #[test]
    fn test_no_comments() {
        let input = "<root><child attr=\"value\">text</child></root>";
        let expected = "<root><child attr=\"value\">text</child></root>";
        assert_eq!(xml_type_remover(input), expected);
    }

    #[test]
    fn test_html_comment_in_script() {
        let input = r#"<script>var x = "<!-- comment -->"; console.log(x);</script>"#;
        let expected = r#"<script>var x = "<!-- comment -->"; console.log(x);</script>"#;
        assert_eq!(xml_type_remover(input), expected);
    }

    #[test]
    fn test_html_comment_in_style() {
        let input =
            r#"<style> body { color: red;  } <!-- html comment --></style>"#;
        let expected = r#"<style> body { color: red;  } </style>"#;
        assert_eq!(xml_type_remover(input), expected);
    }


    #[test]
    fn test_nested_quotes_in_attributes() {
        let input = r#"<a href="https://example.com?q=<!-- test -->">link</a>"#;
        let expected = r#"<a href="https://example.com?q=<!-- test -->">link</a>"#;
        assert_eq!(xml_type_remover(input), expected);
    }
    
    #[test]
    fn test_mixed_single_double_quotes() {
        let input = r#"<root attr='value "with" quotes'><!-- comment --></root>"#;
        let expected = r#"<root attr='value "with" quotes'></root>"#;
        assert_eq!(xml_type_remover(input), expected);
    }

    #[test]
    fn test_processing_instruction() {
        let input = r#"<?xml version="1.0"?><!-- comment --><root></root>"#;
        let expected = r#"<?xml version="1.0"?><root></root>"#;
        assert_eq!(xml_type_remover(input), expected);
    }

    #[test]
    fn test_doctype_preserved() {
        let input = r#"<!DOCTYPE html><!-- comment --><html></html>"#;
        let expected = r#"<!DOCTYPE html><html></html>"#;
        assert_eq!(xml_type_remover(input), expected);
    }

    #[test]
    fn test_complex_nested_structure() {
        let input = r#"<!-- outer --><div attr="<!-- inner -->"><!-- mid --><script>var s = "<!-- script -->";</script><!-- end --></div>"#;
        let expected =
            r#"<div attr="<!-- inner -->"><script>var s = "<!-- script -->";</script></div>"#;
        assert_eq!(xml_type_remover(input), expected);
    }

    #[test]
    fn test_cdata_with_closing_sequence() {
        let input = r#"<root><![CDATA[text with --> sequence inside]]></root>"#;
        let expected = r#"<root><![CDATA[text with --> sequence inside]]></root>"#;
        assert_eq!(xml_type_remover(input), expected);
    }

    #[test]
    fn test_script_with_cdata() {
        let input = r#"<script><![CDATA[var x = "<!-- not removed -->"; ]]></script>"#;
        let expected = r#"<script><![CDATA[var x = "<!-- not removed -->"; ]]></script>"#;
        assert_eq!(xml_type_remover(input), expected);
    }

    #[test]
    fn test_content_after_script_tag() {
        let input = r#"<script><!-- comment --></script><p>After script</p>"#;
        let expected = r#"<script></script><p>After script</p>"#;
        assert_eq!(xml_type_remover(input), expected);
    }

    #[test]
    fn test_content_after_style_tag() {
        let input = r#"<style><!-- comment --></style><p>After style</p>"#;
        let expected = r#"<style></style><p>After style</p>"#;
        assert_eq!(xml_type_remover(input), expected);
    }

    #[test]
    fn test_greedy_comment_consumption() {
        let input = r#"<!-- comment1 <!-- nested --> text --><root></root>"#;

        let expected = " text --><root></root>";
        assert_eq!(xml_type_remover(input), expected);
    }

    #[test]
    fn test_multiple_scripts_with_comments() {
        let input = r#"<script><!-- c1 --></script><!-- c2 --><script><!-- c3 --></script>"#;
        let expected = r#"<script></script><script></script>"#;
        assert_eq!(xml_type_remover(input), expected);
    }
}
