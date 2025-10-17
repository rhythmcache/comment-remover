use clap::Parser;
use comment_remover::{
    remove_c_type_comments, remove_go_comments, remove_hash_comments_basic, remove_js_comments,
    remove_python_comments, remove_rust_comments, remove_shell_comments, xml_type_remover,
};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;
use std::process;
#[derive(Debug, Clone, Copy, PartialEq)]
enum Language {
    C,
    Shell,
    Python,
    HashBasic,
    Xml,
    Js,
    Go,
    Rust,
}
#[derive(Parser, Debug)]
#[command(name = "comment_remover")]
#[command(about = "Remove comments from source code files", long_about = None)]
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
fn parse_language(s: &str) -> Option<Language> {
    match s.to_lowercase().as_str() {
        "c" | "c++" | "cpp" | "java" | "csharp" | "cs" | "swift" => Some(Language::C),
        "rs" | "rust" => Some(Language::Rust),
        "go" | "golang" => Some(Language::Go),
        "javascript" | "typescript" | "js" | "ts" => Some(Language::Js),
        "shell" | "sh" | "bash" | "zsh" => Some(Language::Shell),
        "python" | "py" => Some(Language::Python),
        "hash" | "basic" => Some(Language::HashBasic),
        "xml" | "html" => Some(Language::Xml),
        _ => None,
    }
}
fn detect_language(path: &str) -> Option<Language> {
    let path_obj = Path::new(path);
    let ext = path_obj.extension()?.to_str()?.to_lowercase();
    match ext.as_str() {
        "c" | "h" | "cpp" | "cc" | "cxx" | "hpp" | "hxx" | "java" | "cs" | "swift" | "kt"
        | "scala" | "m" | "mm" => Some(Language::C),
        "rs" | "rust" => Some(Language::Rust),
        "go" | "golang" => Some(Language::Go),
        "sh" | "bash" | "zsh" | "ksh" => Some(Language::Shell),
        "js" | "ts" | "jsx" | "tsx" => Some(Language::Js),
        "py" | "pyw" => Some(Language::Python),
        "rb" | "pl" | "pm" | "r" | "yaml" | "yml" | "toml" | "conf" | "cfg" | "ini" | "mk"
        | "makefile" => Some(Language::HashBasic),
        "xml" | "html" | "htm" => Some(Language::Xml),
        _ => None,
    }
}
fn remove_comments(input: &str, lang: Language) -> String {
    match lang {
        Language::C => remove_c_type_comments(input),
        Language::Shell => remove_shell_comments(input),
        Language::Python => remove_python_comments(input),
        Language::HashBasic => remove_hash_comments_basic(input),
        Language::Xml => xml_type_remover(input),
        Language::Js => remove_js_comments(input),
        Language::Go => remove_go_comments(input),
        Language::Rust => remove_rust_comments(input),
    }
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
    language_override: Option<Language>,
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
        detect_language(file_path)
            .ok_or_else(|| format!("'{}': unsupported file extension", file_path))?
    };
    let input_content = fs::read_to_string(file_path)
        .map_err(|e| format!("Error reading '{}': {}", file_path, e))?;
    let mut output_content = remove_comments(&input_content, language);
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
            parse_language(lang_str).unwrap_or_else(|| {
                eprintln!("Error: Unknown language: {}", lang_str);
                process::exit(1);
            })
        } else {
            eprintln!("Warning: No language specified for stdin, using C-style");
            Language::C
        };
        let mut output_content = remove_comments(&buffer, language);
        if let Some(max_newlines) = args.collapse_whitespace {
            output_content = collapse_whitespace(&output_content, max_newlines);
        }
        print!("{}", output_content);
        io::stdout().flush().unwrap();
        return;
    }
    let language_override = if let Some(lang_str) = &args.language {
        Some(parse_language(lang_str).unwrap_or_else(|| {
            eprintln!("Error: Unknown language: {}", lang_str);
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
