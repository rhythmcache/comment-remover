use clap::Parser;
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;
use std::process;
use tree_sitter::{Language, Parser as TSParser, Query, QueryCursor, StreamingIterator};
#[derive(Debug, Clone, Copy, PartialEq)]
enum TreeSitterLanguage {
    #[cfg(feature = "bash")]
    Bash,
    #[cfg(feature = "c")]
    C,
    #[cfg(feature = "c-sharp")]
    CSharp,
    #[cfg(feature = "cpp")]
    Cpp,
    #[cfg(feature = "css")]
    Css,
    #[cfg(feature = "go")]
    Go,
    #[cfg(feature = "haskell")]
    Haskell,
    #[cfg(feature = "html")]
    Html,
    #[cfg(feature = "java")]
    Java,
    #[cfg(feature = "javascript")]
    JavaScript,
    #[cfg(feature = "lua")]
    Lua,
    #[cfg(feature = "php")]
    Php,
    #[cfg(feature = "python")]
    Python,
    #[cfg(feature = "ruby")]
    Ruby,
    #[cfg(feature = "rust-lang")]
    Rust,
    #[cfg(feature = "scala")]
    Scala,
    #[cfg(feature = "swift")]
    Swift,
    #[cfg(feature = "typescript")]
    TypeScript,
}
impl TreeSitterLanguage {
    fn get_language(&self) -> Language {
        match self {
            #[cfg(feature = "bash")]
            TreeSitterLanguage::Bash => tree_sitter_bash::LANGUAGE.into(),
            #[cfg(feature = "c")]
            TreeSitterLanguage::C => tree_sitter_c::LANGUAGE.into(),
            #[cfg(feature = "c-sharp")]
            TreeSitterLanguage::CSharp => tree_sitter_c_sharp::LANGUAGE.into(),
            #[cfg(feature = "cpp")]
            TreeSitterLanguage::Cpp => tree_sitter_cpp::LANGUAGE.into(),
            #[cfg(feature = "css")]
            TreeSitterLanguage::Css => tree_sitter_css::LANGUAGE.into(),
            #[cfg(feature = "go")]
            TreeSitterLanguage::Go => tree_sitter_go::LANGUAGE.into(),
            #[cfg(feature = "haskell")]
            TreeSitterLanguage::Haskell => tree_sitter_haskell::LANGUAGE.into(),
            #[cfg(feature = "html")]
            TreeSitterLanguage::Html => tree_sitter_html::LANGUAGE.into(),
            #[cfg(feature = "java")]
            TreeSitterLanguage::Java => tree_sitter_java::LANGUAGE.into(),
            #[cfg(feature = "javascript")]
            TreeSitterLanguage::JavaScript => tree_sitter_javascript::LANGUAGE.into(),
            #[cfg(feature = "lua")]
            TreeSitterLanguage::Lua => tree_sitter_lua::LANGUAGE.into(),
            #[cfg(feature = "php")]
            TreeSitterLanguage::Php => tree_sitter_php::LANGUAGE_PHP.into(),
            #[cfg(feature = "python")]
            TreeSitterLanguage::Python => tree_sitter_python::LANGUAGE.into(),
            #[cfg(feature = "ruby")]
            TreeSitterLanguage::Ruby => tree_sitter_ruby::LANGUAGE.into(),
            #[cfg(feature = "rust-lang")]
            TreeSitterLanguage::Rust => tree_sitter_rust::LANGUAGE.into(),
            #[cfg(feature = "scala")]
            TreeSitterLanguage::Scala => tree_sitter_scala::LANGUAGE.into(),
            #[cfg(feature = "swift")]
            TreeSitterLanguage::Swift => tree_sitter_swift::LANGUAGE.into(),
            #[cfg(feature = "typescript")]
            TreeSitterLanguage::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        }
    }
}
#[derive(Parser, Debug)]
#[command(name = "comment_remover")]
#[command(about = "Remove comments from source code files using tree-sitter", long_about = None)]
struct Args {
    #[arg(value_name = "FILES")]
    files: Vec<String>,
    #[arg(short, long, value_name = "LANG")]
    language: Option<String>,
    #[arg(short, long)]
    in_place: bool,
    #[arg(short, long, value_name = "N")]
    collapse_whitespace: Option<usize>,
    #[arg(short, long)]
    force: bool,
}
fn parse_language(s: &str) -> Result<TreeSitterLanguage, String> {
    match s.to_lowercase().as_str() {
        #[cfg(feature = "bash")]
        "bash" | "sh" => Ok(TreeSitterLanguage::Bash),
        #[cfg(feature = "c")]
        "c" => Ok(TreeSitterLanguage::C),
        #[cfg(feature = "c-sharp")]
        "c#" | "csharp" | "cs" => Ok(TreeSitterLanguage::CSharp),
        #[cfg(feature = "cpp")]
        "c++" | "cpp" | "cc" | "cxx" => Ok(TreeSitterLanguage::Cpp),
        #[cfg(feature = "css")]
        "css" => Ok(TreeSitterLanguage::Css),
        #[cfg(feature = "go")]
        "go" | "golang" => Ok(TreeSitterLanguage::Go),
        #[cfg(feature = "haskell")]
        "haskell" | "hs" => Ok(TreeSitterLanguage::Haskell),
        #[cfg(feature = "html")]
        "html" | "htm" => Ok(TreeSitterLanguage::Html),
        #[cfg(feature = "java")]
        "java" => Ok(TreeSitterLanguage::Java),
        #[cfg(feature = "javascript")]
        "javascript" | "js" => Ok(TreeSitterLanguage::JavaScript),
        #[cfg(feature = "lua")]
        "lua" => Ok(TreeSitterLanguage::Lua),
        #[cfg(feature = "php")]
        "php" => Ok(TreeSitterLanguage::Php),
        #[cfg(feature = "python")]
        "python" | "py" => Ok(TreeSitterLanguage::Python),
        #[cfg(feature = "ruby")]
        "ruby" | "rb" => Ok(TreeSitterLanguage::Ruby),
        #[cfg(feature = "rust-lang")]
        "rust" | "rs" => Ok(TreeSitterLanguage::Rust),
        #[cfg(feature = "scala")]
        "scala" => Ok(TreeSitterLanguage::Scala),
        #[cfg(feature = "swift")]
        "swift" => Ok(TreeSitterLanguage::Swift),
        #[cfg(feature = "typescript")]
        "typescript" | "ts" => Ok(TreeSitterLanguage::TypeScript),
        _ => Err(format!(
            "Language '{}' is not supported or not compiled in this build",
            s
        )),
    }
}
fn detect_language(path: &str) -> Option<TreeSitterLanguage> {
    let path_obj = Path::new(path);
    let ext = path_obj.extension()?.to_str()?.to_lowercase();
    match ext.as_str() {
        #[cfg(feature = "bash")]
        "sh" | "bash" => Some(TreeSitterLanguage::Bash),
        #[cfg(feature = "c")]
        "c" | "h" => Some(TreeSitterLanguage::C),
        #[cfg(feature = "c-sharp")]
        "cs" => Some(TreeSitterLanguage::CSharp),
        #[cfg(feature = "cpp")]
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" | "c++" => Some(TreeSitterLanguage::Cpp),
        #[cfg(feature = "css")]
        "css" => Some(TreeSitterLanguage::Css),
        #[cfg(feature = "go")]
        "go" => Some(TreeSitterLanguage::Go),
        #[cfg(feature = "haskell")]
        "hs" => Some(TreeSitterLanguage::Haskell),
        #[cfg(feature = "html")]
        "html" | "htm" => Some(TreeSitterLanguage::Html),
        #[cfg(feature = "java")]
        "java" => Some(TreeSitterLanguage::Java),
        #[cfg(feature = "javascript")]
        "js" | "jsx" | "mjs" | "cjs" => Some(TreeSitterLanguage::JavaScript),
        #[cfg(feature = "lua")]
        "lua" => Some(TreeSitterLanguage::Lua),
        #[cfg(feature = "php")]
        "php" => Some(TreeSitterLanguage::Php),
        #[cfg(feature = "python")]
        "py" | "pyw" => Some(TreeSitterLanguage::Python),
        #[cfg(feature = "ruby")]
        "rb" => Some(TreeSitterLanguage::Ruby),
        #[cfg(feature = "rust-lang")]
        "rs" => Some(TreeSitterLanguage::Rust),
        #[cfg(feature = "scala")]
        "scala" => Some(TreeSitterLanguage::Scala),
        #[cfg(feature = "swift")]
        "swift" => Some(TreeSitterLanguage::Swift),
        #[cfg(feature = "typescript")]
        "ts" | "tsx" | "mts" | "cts" => Some(TreeSitterLanguage::TypeScript),
        _ => None,
    }
}
fn get_supported_languages() -> String {
    let mut langs = Vec::new();
    #[cfg(feature = "bash")]
    langs.push("bash");
    #[cfg(feature = "c")]
    langs.push("c");
    #[cfg(feature = "c-sharp")]
    langs.push("c#");
    #[cfg(feature = "cpp")]
    langs.push("c++");
    #[cfg(feature = "css")]
    langs.push("css");
    #[cfg(feature = "go")]
    langs.push("go");
    #[cfg(feature = "haskell")]
    langs.push("haskell");
    #[cfg(feature = "html")]
    langs.push("html");
    #[cfg(feature = "java")]
    langs.push("java");
    #[cfg(feature = "javascript")]
    langs.push("javascript");
    #[cfg(feature = "lua")]
    langs.push("lua");
    #[cfg(feature = "php")]
    langs.push("php");
    #[cfg(feature = "python")]
    langs.push("python");
    #[cfg(feature = "ruby")]
    langs.push("ruby");
    #[cfg(feature = "rust-lang")]
    langs.push("rust");
    #[cfg(feature = "scala")]
    langs.push("scala");
    #[cfg(feature = "swift")]
    langs.push("swift");
    #[cfg(feature = "typescript")]
    langs.push("typescript");
    if langs.is_empty() {
        "none (rebuild with language features enabled)".to_string()
    } else {
        langs.join(", ")
    }
}
fn remove_comments_treesitter(input: &str, language: TreeSitterLanguage) -> Result<String, String> {
    let ts_language = language.get_language();
    let mut parser = TSParser::new();
    parser
        .set_language(&ts_language)
        .map_err(|e| format!("Error loading grammar: {}", e))?;
    let tree = parser
        .parse(input, None)
        .ok_or_else(|| "Error parsing input".to_string())?;
    let query_str = match language {
        #[cfg(feature = "bash")]
        TreeSitterLanguage::Bash => "(comment) @comment",
        #[cfg(feature = "c")]
        TreeSitterLanguage::C => "(comment) @comment",
        #[cfg(feature = "c-sharp")]
        TreeSitterLanguage::CSharp => "(comment) @comment",
        #[cfg(feature = "cpp")]
        TreeSitterLanguage::Cpp => "(comment) @comment",
        #[cfg(feature = "css")]
        TreeSitterLanguage::Css => "(comment) @comment",
        #[cfg(feature = "go")]
        TreeSitterLanguage::Go => "(comment) @comment",
        #[cfg(feature = "haskell")]
        TreeSitterLanguage::Haskell => "(comment) @comment",
        #[cfg(feature = "html")]
        TreeSitterLanguage::Html => "(comment) @comment",
        #[cfg(feature = "java")]
        TreeSitterLanguage::Java => "(line_comment) @comment (block_comment) @comment",
        #[cfg(feature = "javascript")]
        TreeSitterLanguage::JavaScript => "(comment) @comment",
        #[cfg(feature = "lua")]
        TreeSitterLanguage::Lua => "(comment) @comment",
        #[cfg(feature = "php")]
        TreeSitterLanguage::Php => "(comment) @comment",
        #[cfg(feature = "python")]
        TreeSitterLanguage::Python => "(comment) @comment",
        #[cfg(feature = "ruby")]
        TreeSitterLanguage::Ruby => "(comment) @comment",
        #[cfg(feature = "rust-lang")]
        TreeSitterLanguage::Rust => "(line_comment) @comment (block_comment) @comment",
        #[cfg(feature = "scala")]
        TreeSitterLanguage::Scala => "(comment) @comment",
        #[cfg(feature = "swift")]
        TreeSitterLanguage::Swift => "(comment) @comment",
        #[cfg(feature = "typescript")]
        TreeSitterLanguage::TypeScript => "(comment) @comment",
    };
    if query_str.is_empty() {
        return Ok(input.to_string());
    }
    let query =
        Query::new(&ts_language, query_str).map_err(|e| format!("Error creating query: {}", e))?;
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), input.as_bytes());
    let mut comment_ranges = Vec::new();
    while let Some(m) = matches.next() {
        for capture in m.captures {
            comment_ranges.push(capture.node.byte_range());
        }
    }
    comment_ranges.sort_by_key(|r| r.start);
    let mut result = String::with_capacity(input.len());
    let mut last_pos = 0;
    for range in comment_ranges {
        result.push_str(&input[last_pos..range.start]);
        for ch in input[range.clone()].chars() {
            if ch == '\n' {
                result.push('\n');
            }
        }
        last_pos = range.end;
    }
    result.push_str(&input[last_pos..]);
    Ok(result)
}
fn collapse_whitespace(input: &str, max_newlines: usize) -> String {
    let lines: Vec<&str> = input.lines().collect();
    let mut result = String::with_capacity(input.len());
    let mut consecutive_empty = 0;
    for (idx, line) in lines.iter().enumerate() {
        let is_empty = line.trim().is_empty();
        if is_empty {
            consecutive_empty += 1;
            if consecutive_empty <= max_newlines {
                if idx > 0 {
                    result.push('\n');
                }
                result.push_str(line);
            }
        } else {
            consecutive_empty = 0;
            if idx > 0 {
                result.push('\n');
            }
            result.push_str(line);
        }
    }
    if input.ends_with('\n') && !result.ends_with('\n') {
        result.push('\n');
    }
    result
}
fn process_single_file(
    file_path: &str,
    language_override: Option<TreeSitterLanguage>,
    collapse: Option<usize>,
) -> Result<String, String> {
    let metadata =
        fs::metadata(file_path).map_err(|e| format!("Cannot access '{}': {}", file_path, e))?;
    if metadata.is_dir() {
        return Err(format!("'{}' is a directory, not a file", file_path));
    }
    let language = if let Some(lang) = language_override {
        lang
    } else {
        detect_language(file_path).ok_or_else(|| {
            format!(
                "'{}': unsupported or unavailable language for this file",
                file_path
            )
        })?
    };
    let input_content = fs::read_to_string(file_path)
        .map_err(|e| format!("Error reading '{}': {}", file_path, e))?;
    let mut output_content = remove_comments_treesitter(&input_content, language)
        .map_err(|e| format!("Error processing '{}': {}", file_path, e))?;
    if let Some(max_newlines) = collapse {
        output_content = collapse_whitespace(&output_content, max_newlines);
    }
    Ok(output_content)
}
fn main() {
    let args = Args::parse();
    if args.files.is_empty() {
        if args.in_place {
            eprintln!("Error: --in-place requires at least one input file");
            process::exit(1);
        }
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer).unwrap_or_else(|e| {
            eprintln!("Error reading stdin: {}", e);
            process::exit(1);
        });
        let language = if let Some(lang_str) = &args.language {
            parse_language(lang_str).unwrap_or_else(|e| {
                eprintln!("Error: {}", e);
                eprintln!(
                    "Supported languages in this build: {}",
                    get_supported_languages()
                );
                process::exit(1);
            })
        } else {
            eprintln!("Error: Language must be specified for stdin input (use -l/--language)");
            eprintln!("Supported languages: {}", get_supported_languages());
            process::exit(1);
        };
        let mut output_content =
            remove_comments_treesitter(&buffer, language).unwrap_or_else(|e| {
                eprintln!("Error: {}", e);
                process::exit(1);
            });
        if let Some(max_newlines) = args.collapse_whitespace {
            output_content = collapse_whitespace(&output_content, max_newlines);
        }
        print!("{}", output_content);
        io::stdout().flush().unwrap();
        return;
    }
    let language_override = if let Some(lang_str) = &args.language {
        Some(parse_language(lang_str).unwrap_or_else(|e| {
            eprintln!("Error: {}", e);
            eprintln!(
                "Supported languages in this build: {}",
                get_supported_languages()
            );
            process::exit(1);
        }))
    } else {
        None
    };
    let mut failed_files = Vec::new();
    let mut processed_count = 0;
    for file_path in &args.files {
        match process_single_file(file_path, language_override, args.collapse_whitespace) {
            Ok(output_content) => {
                if args.in_place {
                    if let Err(e) = fs::write(file_path, &output_content) {
                        eprintln!("Error writing to '{}': {}", file_path, e);
                        failed_files.push(file_path.clone());
                        if !args.force {
                            process::exit(1);
                        }
                    } else {
                        processed_count += 1;
                    }
                } else {
                    if args.files.len() > 1 {
                        eprintln!(
                            "Error: Cannot output multiple files to stdout without --in-place"
                        );
                        process::exit(1);
                    }
                    print!("{}", output_content);
                    io::stdout().flush().unwrap();
                    processed_count += 1;
                }
            }
            Err(err_msg) => {
                eprintln!("Error: {}", err_msg);
                failed_files.push(file_path.clone());
                if !args.force {
                    process::exit(1);
                }
            }
        }
    }
    if !failed_files.is_empty() {
        eprintln!(
            "\nProcessed: {}, Failed: {}",
            processed_count,
            failed_files.len()
        );
        process::exit(1);
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[cfg(feature = "python")]
    fn test_python_single_line_comment() {
        let input = "# This is a comment\nprint('hello')";
        let result = remove_comments_treesitter(input, TreeSitterLanguage::Python).unwrap();
        assert!(!result.contains("# This is a comment"));
        assert!(result.contains("print('hello')"));
    }
    #[test]
    #[cfg(feature = "python")]
    fn test_python_inline_comment() {
        let input = "x = 5  # set x to 5\nprint(x)";
        let result = remove_comments_treesitter(input, TreeSitterLanguage::Python).unwrap();
        assert!(!result.contains("# set x to 5"));
        assert!(result.contains("x = 5"));
    }
    #[test]
    #[cfg(feature = "python")]
    fn test_python_multiple_comments() {
        let input = "# Comment 1\n# Comment 2\ncode()\n# Comment 3";
        let result = remove_comments_treesitter(input, TreeSitterLanguage::Python).unwrap();
        assert!(!result.contains("Comment"));
        assert!(result.contains("code()"));
    }
    #[test]
    #[cfg(feature = "python")]
    fn test_python_preserves_strings_with_hash() {
        let input = "text = '# not a comment'\nprint(text)";
        let result = remove_comments_treesitter(input, TreeSitterLanguage::Python).unwrap();
        assert!(result.contains("# not a comment"));
    }
    #[test]
    #[cfg(feature = "rust-lang")]
    fn test_rust_line_comment() {
        let input = "// This is a comment\nfn main() {}";
        let result = remove_comments_treesitter(input, TreeSitterLanguage::Rust).unwrap();
        assert!(!result.contains("// This is a comment"));
        assert!(result.contains("fn main()"));
    }
    #[test]
    #[cfg(feature = "rust-lang")]
    fn test_rust_block_comment() {
        let input = "/* Block comment */\nfn test() {}";
        let result = remove_comments_treesitter(input, TreeSitterLanguage::Rust).unwrap();
        assert!(!result.contains("/* Block comment */"));
        assert!(result.contains("fn test()"));
    }
    #[test]
    #[cfg(feature = "rust-lang")]
    fn test_rust_multiline_block_comment() {
        let input = "/*\n * Multi-line\n * comment\n */\nlet x = 5;";
        let result = remove_comments_treesitter(input, TreeSitterLanguage::Rust).unwrap();
        assert!(!result.contains("Multi-line"));
        assert!(result.contains("let x = 5"));
    }
    #[test]
    #[cfg(feature = "rust-lang")]
    fn test_rust_inline_comment() {
        let input = "let x = 5; // inline comment";
        let result = remove_comments_treesitter(input, TreeSitterLanguage::Rust).unwrap();
        assert!(!result.contains("// inline comment"));
        assert!(result.contains("let x = 5;"));
    }
    #[test]
    #[cfg(feature = "javascript")]
    fn test_javascript_single_line() {
        let input = "// Single line comment\nconsole.log('test');";
        let result = remove_comments_treesitter(input, TreeSitterLanguage::JavaScript).unwrap();
        assert!(!result.contains("// Single line"));
        assert!(result.contains("console.log"));
    }
    #[test]
    #[cfg(feature = "javascript")]
    fn test_javascript_multiline() {
        let input = "/* Multi\n   line\n   comment */\nvar x = 1;";
        let result = remove_comments_treesitter(input, TreeSitterLanguage::JavaScript).unwrap();
        assert!(!result.contains("Multi"));
        assert!(result.contains("var x = 1"));
    }
    #[test]
    #[cfg(feature = "javascript")]
    fn test_javascript_jsdoc_removed() {
        let input = "/**\n * JSDoc comment\n */\nfunction test() {}";
        let result = remove_comments_treesitter(input, TreeSitterLanguage::JavaScript).unwrap();
        assert!(!result.contains("JSDoc"));
        assert!(result.contains("function test()"));
    }
    #[test]
    #[cfg(feature = "c")]
    fn test_c_single_line() {
        let input = "// C++ style comment\nint x = 5;";
        let result = remove_comments_treesitter(input, TreeSitterLanguage::C).unwrap();
        assert!(!result.contains("// C++ style"));
        assert!(result.contains("int x = 5"));
    }
    #[test]
    #[cfg(feature = "c")]
    fn test_c_block_comment() {
        let input = "/* C style comment */\nint main() { return 0; }";
        let result = remove_comments_treesitter(input, TreeSitterLanguage::C).unwrap();
        assert!(!result.contains("/* C style"));
        assert!(result.contains("int main()"));
    }
    #[test]
    #[cfg(feature = "cpp")]
    fn test_cpp_comments() {
        let input = "// C++ comment\n/* Block */\nint main() {}";
        let result = remove_comments_treesitter(input, TreeSitterLanguage::Cpp).unwrap();
        assert!(!result.contains("// C++"));
        assert!(!result.contains("/* Block */"));
        assert!(result.contains("int main()"));
    }
    #[test]
    #[cfg(feature = "java")]
    fn test_java_comments() {
        let input = "// Java comment\npublic class Test {}";
        let result = remove_comments_treesitter(input, TreeSitterLanguage::Java).unwrap();
        assert!(!result.contains("// Java"));
        assert!(result.contains("public class Test"));
    }
    #[test]
    #[cfg(feature = "java")]
    fn test_java_javadoc() {
        let input = "/**\n * Javadoc\n */\npublic void method() {}";
        let result = remove_comments_treesitter(input, TreeSitterLanguage::Java).unwrap();
        assert!(!result.contains("Javadoc"));
        assert!(result.contains("public void method()"));
    }
    #[test]
    #[cfg(feature = "go")]
    fn test_go_comments() {
        let input = "// Go comment\nfunc main() {}";
        let result = remove_comments_treesitter(input, TreeSitterLanguage::Go).unwrap();
        assert!(!result.contains("// Go comment"));
        assert!(result.contains("func main()"));
    }
    #[test]
    #[cfg(feature = "go")]
    fn test_go_block_comment() {
        let input = "/* Block comment in Go */\npackage main";
        let result = remove_comments_treesitter(input, TreeSitterLanguage::Go).unwrap();
        assert!(!result.contains("/* Block"));
        assert!(result.contains("package main"));
    }
    #[test]
    #[cfg(feature = "ruby")]
    fn test_ruby_single_line() {
        let input = "# Ruby comment\nputs 'hello'";
        let result = remove_comments_treesitter(input, TreeSitterLanguage::Ruby).unwrap();
        assert!(!result.contains("# Ruby comment"));
        assert!(result.contains("puts 'hello'"));
    }
    #[test]
    #[cfg(feature = "ruby")]
    fn test_ruby_multiline() {
        let input = "=begin\nMulti-line\ncomment\n=end\nputs 'test'";
        let result = remove_comments_treesitter(input, TreeSitterLanguage::Ruby).unwrap();
        assert!(!result.contains("Multi-line"));
        assert!(result.contains("puts 'test'"));
    }
    #[test]
    #[cfg(feature = "php")]
    fn test_php_comments() {
        let input = "<?php\n// PHP comment\n# Hash comment\n/* Block */\necho 'test';\n?>";
        let result = remove_comments_treesitter(input, TreeSitterLanguage::Php).unwrap();
        assert!(!result.contains("// PHP"));
        assert!(!result.contains("# Hash"));
        assert!(!result.contains("/* Block */"));
        assert!(result.contains("echo 'test'"));
    }
    #[test]
    #[cfg(feature = "bash")]
    fn test_bash_comments() {
        let input = "#!/bin/bash\n# This is a comment\necho 'hello'";
        let result = remove_comments_treesitter(input, TreeSitterLanguage::Bash).unwrap();
        assert!(!result.contains("# This is a comment"));
        assert!(result.contains("echo 'hello'"));
    }
    #[test]
    #[cfg(feature = "css")]
    fn test_css_comments() {
        let input = "/* CSS comment */\nbody { color: red; }";
        let result = remove_comments_treesitter(input, TreeSitterLanguage::Css).unwrap();
        assert!(!result.contains("/* CSS comment */"));
        assert!(result.contains("body { color: red; }"));
    }
    #[test]
    #[cfg(feature = "html")]
    fn test_html_comments() {
        let input = "<!-- HTML comment -->\n<div>Content</div>";
        let result = remove_comments_treesitter(input, TreeSitterLanguage::Html).unwrap();
        assert!(!result.contains("<!-- HTML comment -->"));
        assert!(result.contains("<div>Content</div>"));
    }
    #[test]
    #[cfg(feature = "lua")]
    fn test_lua_single_line() {
        let input = "-- Lua comment\nprint('hello')";
        let result = remove_comments_treesitter(input, TreeSitterLanguage::Lua).unwrap();
        assert!(!result.contains("-- Lua comment"));
        assert!(result.contains("print('hello')"));
    }
    #[test]
    #[cfg(feature = "lua")]
    fn test_lua_multiline() {
        let input = "--[[\nMulti-line\ncomment\n]]\nlocal x = 5";
        let result = remove_comments_treesitter(input, TreeSitterLanguage::Lua).unwrap();
        assert!(!result.contains("Multi-line"));
        assert!(result.contains("local x = 5"));
    }
    #[test]
    #[cfg(feature = "haskell")]
    fn test_haskell_single_line() {
        let input = "-- Haskell comment\nmain = print \"hello\"";
        let result = remove_comments_treesitter(input, TreeSitterLanguage::Haskell).unwrap();
        assert!(!result.contains("-- Haskell"));
        assert!(result.contains("main = print"));
    }
    #[test]
    #[cfg(feature = "haskell")]
    fn test_haskell_block() {
        let input = "{- Block comment -}\nfunc x = x + 1";
        let result = remove_comments_treesitter(input, TreeSitterLanguage::Haskell).unwrap();
        assert!(!result.contains("{- Block"));
        assert!(result.contains("func x = x + 1"));
    }
    #[test]
    #[cfg(feature = "swift")]
    fn test_swift_comments() {
        let input = "// Swift comment\nvar x = 5";
        let result = remove_comments_treesitter(input, TreeSitterLanguage::Swift).unwrap();
        assert!(!result.contains("// Swift"));
        assert!(result.contains("var x = 5"));
    }
    #[test]
    #[cfg(feature = "scala")]
    fn test_scala_comments() {
        let input = "// Scala comment\nval x = 5";
        let result = remove_comments_treesitter(input, TreeSitterLanguage::Scala).unwrap();
        assert!(!result.contains("// Scala"));
        assert!(result.contains("val x = 5"));
    }
    #[test]
    #[cfg(feature = "typescript")]
    fn test_typescript_comments() {
        let input = "// TypeScript comment\nlet x: number = 5;";
        let result = remove_comments_treesitter(input, TreeSitterLanguage::TypeScript).unwrap();
        assert!(!result.contains("// TypeScript"));
        assert!(result.contains("let x: number = 5"));
    }
    #[test]
    #[cfg(feature = "c-sharp")]
    fn test_csharp_comments() {
        let input = "// C# comment\nint x = 5;";
        let result = remove_comments_treesitter(input, TreeSitterLanguage::CSharp).unwrap();
        assert!(!result.contains("// C#"));
        assert!(result.contains("int x = 5"));
    }
    #[test]
    #[cfg(feature = "c-sharp")]
    fn test_csharp_xml_doc() {
        let input = "/// <summary>XML doc</summary>\npublic void Method() {}";
        let result = remove_comments_treesitter(input, TreeSitterLanguage::CSharp).unwrap();
        assert!(!result.contains("XML doc"));
        assert!(result.contains("public void Method()"));
    }
    #[test]
    #[cfg(feature = "python")]
    fn test_preserves_newlines_in_comments() {
        let input = "# Comment line 1\n# Comment line 2\ncode()";
        let result = remove_comments_treesitter(input, TreeSitterLanguage::Python).unwrap();
        assert_eq!(result.matches('\n').count(), input.matches('\n').count());
    }
    #[test]
    #[cfg(feature = "rust-lang")]
    fn test_nested_block_comments_rust() {
        let input = "/* outer /* inner */ outer */\nlet x = 1;";
        let result = remove_comments_treesitter(input, TreeSitterLanguage::Rust).unwrap();
        assert!(!result.contains("outer"));
        assert!(!result.contains("inner"));
        assert!(result.contains("let x = 1"));
    }
    #[test]
    #[cfg(feature = "javascript")]
    fn test_comment_like_strings_preserved() {
        let input = "var url = 'http://example.com';\nvar comment = '// not a comment';";
        let result = remove_comments_treesitter(input, TreeSitterLanguage::JavaScript).unwrap();
        assert!(result.contains("http://example.com"));
        assert!(result.contains("// not a comment"));
    }
}
