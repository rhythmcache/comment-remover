use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;
use std::process;

use comment_remover::{
    remove_c_type_comments, remove_hash_comments_basic, remove_python_comments,
    remove_shell_comments,
};

#[derive(Debug, Clone, Copy, PartialEq)]
enum Language {
    C,
    Shell,
    Python,
    HashBasic,
}

struct Config {
    input: Option<String>,
    output: Option<String>,
    language: Option<Language>,
    collapse_whitespace: bool,
    max_newlines: usize,
    in_place: bool,
}

impl Config {
    fn new() -> Self {
        Config {
            input: None,
            output: None,
            language: None,
            collapse_whitespace: false,
            max_newlines: usize::MAX,
            in_place: false,
        }
    }
}

fn print_usage(program_name: &str) {
    eprintln!("Usage: {} [OPTIONS] [INPUT] [OUTPUT]", program_name);
    eprintln!();
    eprintln!("Remove comments from source code files.");
    eprintln!();
    eprintln!("ARGUMENTS:");
    eprintln!("  [INPUT]     Input file (omit or use '-' for stdin)");
    eprintln!("  [OUTPUT]    Output file (omit for stdout)");
    eprintln!();
    eprintln!("OPTIONS:");
    eprintln!("  -l, --language LANG    Force language detection (c, shell, python, hash)");
    eprintln!("                         c: C/C++/Java/JavaScript/Rust style (// and /* */)");
    eprintln!("                         shell: Bash/Zsh with heredoc support");
    eprintln!("                         python: Python with docstrings");
    eprintln!("                         hash: Basic # comment removal");
    eprintln!("  -i, --in-place         Modify file in place");
    eprintln!("  -c, --collapse-whitespace");
    eprintln!("                         Remove excessive blank lines");
    eprintln!("  -s, --space N          Maximum consecutive newlines (implies -c)");
    eprintln!("  -h, --help             Show this help message");
}

fn print_help(program_name: &str) {
    print_usage(program_name);
    eprintln!();
    eprintln!("SUPPORTED EXTENSIONS:");
    eprintln!(
        "  C-style:    .c, .h, .cpp, .cc, .cxx, .hpp, .java, .js, .ts, .rs, .go, .cs, .swift"
    );
    eprintln!("  Shell:      .sh, .bash, .zsh");
    eprintln!("  Python:     .py");
    eprintln!("  Hash-basic: .rb, .pl, .r, .yaml, .yml, .toml, .conf, .cfg");
    eprintln!();
    eprintln!("EXAMPLES:");
    eprintln!(
        "  {} file.c                  # Output to stdout",
        program_name
    );
    eprintln!(
        "  {} file.c output.c         # Write to output.c",
        program_name
    );
    eprintln!(
        "  {} -i file.c               # Modify file.c in place",
        program_name
    );
    eprintln!(
        "  cat file.c | {}            # Read from stdin",
        program_name
    );
    eprintln!(
        "  {} -l python file.txt      # Force Python mode",
        program_name
    );
    eprintln!(
        "  {} -c file.c               # Collapse whitespace",
        program_name
    );
    eprintln!(
        "  {} -s 2 file.c             # Max 2 blank lines",
        program_name
    );
}

fn detect_language(path: &str) -> Option<Language> {
    let path_obj = Path::new(path);
    let ext = path_obj.extension()?.to_str()?.to_lowercase();

    match ext.as_str() {
        "c" | "h" | "cpp" | "cc" | "cxx" | "hpp" | "hxx" | "java" | "js" | "ts" | "jsx" | "tsx"
        | "rs" | "go" | "cs" | "swift" | "kt" | "scala" | "m" | "mm" => Some(Language::C),

        "sh" | "bash" | "zsh" | "ksh" => Some(Language::Shell),

        "py" | "pyw" => Some(Language::Python),

        "rb" | "pl" | "pm" | "r" | "yaml" | "yml" | "toml" | "conf" | "cfg" | "ini" | "mk"
        | "makefile" => Some(Language::HashBasic),

        _ => None,
    }
}

fn parse_language(s: &str) -> Option<Language> {
    match s.to_lowercase().as_str() {
        "c" | "c++" | "cpp" | "java" | "javascript" | "js" | "rust" | "rs" | "go" | "golang"
        | "csharp" | "cs" | "swift" => Some(Language::C),
        "shell" | "sh" | "bash" | "zsh" => Some(Language::Shell),
        "python" | "py" => Some(Language::Python),
        "hash" | "basic" => Some(Language::HashBasic),
        _ => None,
    }
}

fn parse_args() -> Result<Config, String> {
    let args: Vec<String> = env::args().collect();
    let mut config = Config::new();
    let mut positional = Vec::new();
    let mut i = 1;
    let mut parse_options = true;

    let program_name = args
        .get(0)
        .and_then(|p| Path::new(p).file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("comment_remover");

    while i < args.len() {
        let arg = &args[i];

        if parse_options && arg == "--" {
            parse_options = false;
            i += 1;
            continue;
        }

        if parse_options && arg == "-h" || arg == "--help" {
            print_help(program_name);
            process::exit(0);
        } else if parse_options && (arg == "-i" || arg == "--in-place") {
            config.in_place = true;
        } else if parse_options && (arg == "-c" || arg == "--collapse-whitespace") {
            config.collapse_whitespace = true;
        } else if parse_options && (arg == "-l" || arg == "--language") {
            i += 1;
            if i >= args.len() {
                return Err(format!("Missing value for {}", arg));
            }
            config.language = Some(
                parse_language(&args[i]).ok_or_else(|| format!("Unknown language: {}", args[i]))?,
            );
        } else if parse_options && (arg == "-s" || arg == "--space") {
            i += 1;
            if i >= args.len() {
                return Err(format!("Missing value for {}", arg));
            }
            config.max_newlines = args[i]
                .parse()
                .map_err(|_| format!("Invalid number for --space: {}", args[i]))?;
            config.collapse_whitespace = true;
        } else if parse_options && arg.starts_with('-') {
            return Err(format!("Unknown option: {}", arg));
        } else {
            positional.push(arg.clone());
        }

        i += 1;
    }

    if !positional.is_empty() {
        config.input = Some(positional[0].clone());
        if positional.len() > 1 {
            config.output = Some(positional[1].clone());
        }
        if positional.len() > 2 {
            return Err("Too many positional arguments".to_string());
        }
    }

    if config.in_place {
        if config.input.is_none() {
            return Err("--in-place requires an input file".to_string());
        }
        if config.input.as_ref().map(|s| s.as_str()) == Some("-") {
            return Err("--in-place cannot be used with stdin".to_string());
        }
        if config.output.is_some() {
            return Err("--in-place cannot be used with output file".to_string());
        }
    }

    Ok(config)
}

fn remove_comments(input: &str, lang: Language) -> String {
    match lang {
        Language::C => remove_c_type_comments(input),
        Language::Shell => remove_shell_comments(input),
        Language::Python => remove_python_comments(input),
        Language::HashBasic => remove_hash_comments_basic(input),
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
    let config = match parse_args() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {}", e);
            eprintln!();
            let program_name = env::args()
                .next()
                .and_then(|p| {
                    Path::new(&p)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(String::from)
                })
                .unwrap_or_else(|| "comment_remover".to_string());
            print_usage(&program_name);
            process::exit(1);
        }
    };

    let input_content = if let Some(ref input_file) = config.input {
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

    let language = if let Some(lang) = config.language {
        lang
    } else if let Some(ref input_file) = config.input {
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

    if config.collapse_whitespace {
        output_content = collapse_whitespace(&output_content, config.max_newlines);
    }

    if config.in_place {
        let input_file = config.input.as_ref().unwrap();
        fs::write(input_file, &output_content).unwrap_or_else(|e| {
            eprintln!("Error writing to file '{}': {}", input_file, e);
            process::exit(1);
        });
    } else if let Some(ref output_file) = config.output {
        fs::write(output_file, &output_content).unwrap_or_else(|e| {
            eprintln!("Error writing to file '{}': {}", output_file, e);
            process::exit(1);
        });
    } else {
        print!("{}", output_content);
        io::stdout().flush().unwrap();
    }
}
