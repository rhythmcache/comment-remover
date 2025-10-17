use clap::Parser;
use comment_remover::{
    remove_c_type_comments, remove_hash_comments_basic, remove_js_comments, remove_python_comments,
    remove_shell_comments, xml_type_remover,
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
}
#[derive(Parser, Debug)]
#[command(name = "comment_remover")]
#[command(about = "Remove comments from source code files", long_about = None)]
struct Args {
    #[arg(value_name = "INPUT")]
    input: Option<String>,
    #[arg(value_name = "OUTPUT")]
    output: Option<String>,
    #[arg(short, long, value_name = "LANG")]
    language: Option<String>,
    #[arg(short, long)]
    in_place: bool,
    #[arg(short, long, value_name = "N")]
    collapse_whitespace: Option<usize>,
}
fn parse_language(s: &str) -> Option<Language> {
    match s.to_lowercase().as_str() {
        "c" | "c++" | "cpp" | "java" | "rust" | "rs" | "go" | "golang" | "csharp" | "cs"
        | "swift" => Some(Language::C),
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
        "c" | "h" | "cpp" | "cc" | "cxx" | "hpp" | "hxx" | "java" | "rs" | "go" | "cs"
        | "swift" | "kt" | "scala" | "m" | "mm" => Some(Language::C),
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
fn main() {
    let args = Args::parse();
    if args.in_place {
        if args.input.is_none() {
            eprintln!("Error: --in-place requires an input file");
            process::exit(1);
        }
        if args.input.as_ref().map(|s| s.as_str()) == Some("-") {
            eprintln!("Error: --in-place cannot be used with stdin");
            process::exit(1);
        }
        if args.output.is_some() {
            eprintln!("Error: --in-place cannot be used with output file");
            process::exit(1);
        }
    }
    let input_content = if let Some(ref input_file) = args.input {
        if input_file == "-" {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer).unwrap_or_else(|e| {
                eprintln!("Error reading stdin: {}", e);
                process::exit(1);
            });
            buffer
        } else {
            fs::read_to_string(input_file).unwrap_or_else(|e| {
                eprintln!("Error reading file '{}': {}", input_file, e);
                process::exit(1);
            })
        }
    } else {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer).unwrap_or_else(|e| {
            eprintln!("Error reading stdin: {}", e);
            process::exit(1);
        });
        buffer
    };
    let language = if let Some(lang_str) = &args.language {
        parse_language(lang_str).unwrap_or_else(|| {
            eprintln!("Error: Unknown language: {}", lang_str);
            process::exit(1);
        })
    } else if let Some(ref input_file) = args.input {
        if input_file != "-" {
            detect_language(input_file).unwrap_or_else(|| {
                eprintln!("Warning: Could not detect language from extension, using C-style");
                Language::C
            })
        } else {
            eprintln!("Warning: No language specified for stdin, using C-style");
            Language::C
        }
    } else {
        eprintln!("Warning: No language specified for stdin, using C-style");
        Language::C
    };
    let mut output_content = remove_comments(&input_content, language);
    if let Some(max_newlines) = args.collapse_whitespace {
        output_content = collapse_whitespace(&output_content, max_newlines);
    }
    if args.in_place {
        let input_file = args.input.as_ref().unwrap();
        fs::write(input_file, &output_content).unwrap_or_else(|e| {
            eprintln!("Error writing to file '{}': {}", input_file, e);
            process::exit(1);
        });
    } else if let Some(ref output_file) = args.output {
        fs::write(output_file, &output_content).unwrap_or_else(|e| {
            eprintln!("Error writing to file '{}': {}", output_file, e);
            process::exit(1);
        });
    } else {
        print!("{}", output_content);
        io::stdout().flush().unwrap();
    }
}
