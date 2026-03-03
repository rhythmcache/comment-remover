# Comment Remover

[![Crates.io](https://img.shields.io/crates/v/comment-remover.svg)](https://crates.io/crates/comment-remover)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)

**comment-remover** is a fast, accurate command-line tool that strips comments from source code using [tree‑sitter](https://tree-sitter.github.io/).  
It supports **dozens of programming languages**, works on single files or whole directories, and runs in parallel for maximum performance. The binary is named `rmcm`.

Whether you need to clean up code before analysis, minify sources, or simply remove clutter, `rmcm` does the job safely – comments inside strings or literals are never touched.

---

## Features

- [x] **Syntax‑aware removal** – Uses precise tree‑sitter grammars; never mistakes a string for a comment.
- [x] **40+ languages** – Enable only the ones you need (each language is an optional Cargo feature).
- [x] **Parallel processing** – Automatically uses all CPU cores (configurable with `--threads`).
- [x] **Recursive directory traversal** – Process entire source trees with `-r` / `--recursive`.
- [x] **In‑place editing** – Overwrite files directly with `-i` / `--in-place`.
- [x] **Output directory** – Write cleaned files to a separate folder while preserving the relative structure (`--output-dir`).
- [x] **Whitespace collapsing** – Reduce consecutive blank lines to a desired maximum (`-c N`).
- [x] **Diff mode** – See what would change without modifying files (`--diff`).
- [x] **Dry‑run** – Preview which files would be processed (`--dry-run`).
- [x] **JSON output** – Integrate with editors or build tools (`--json`).
- [x] **Configuration file** – Store default options in a TOML file (`--config`).
- [x] **Force continue** – Keep going even if some files fail (`-f` / `--force`).
- [x] **Custom thread count** – Control parallelism (`--threads`).
- [x] **Verbose / quiet logging** – Adjust output detail (`-v`, `-q`).

---

## Installation

### Quick install with `cargo binstall`

If you have [cargo-binstall](https://github.com/cargo-bins/cargo-binstall) installed:

```bash
cargo binstall comment-remover
```

### Install from source with `cargo`

```bash
cargo install comment-remover
```

By default this installs a **minimal set** of languages: C, C++, Rust, JavaScript, Python (the `default` feature).  
To get **all** languages, use `--all-features`:

```bash
cargo install comment-remover --all-features
```

Or select exactly the languages you need (results in a smaller binary):

```bash
# Example: Python and Rust only
cargo install comment-remover --features "python,rust-lang"

# Example: JavaScript, TypeScript, C, and C++
cargo install comment-remover --features "javascript,typescript,c,cpp"
```

After installation, the binary `rmcm` will be available in your `PATH`.

### Build from the repository

```bash
git clone https://github.com/rhythmcache/comment-remover
cd comment-remover
cargo build --release
# binary is at target/release/rmcm
# Optionally install it:
cargo install --path .
```

---

## Usage

```bash
rmcm [OPTIONS] [FILES]...
```

If no files are given, the tool reads from standard input (you **must** provide `--language` in that case).

### Options

| Option                          | Description                                                                                     |
| ------------------------------- | ----------------------------------------------------------------------------------------------- |
| `-l, --language <LANG>`         | Force language (overrides auto‑detection). Required for stdin.                                  |
| `-i, --in-place`                | Edit files in‑place (overwrite original).                                                       |
| `-c, --collapse-whitespace <N>` | Keep at most `N` consecutive blank lines. Use `0` to remove all blank lines.                    |
| `-r, --recursive`               | Process directories recursively.                                                                |
| `--output-dir <DIR>`            | Write output to `DIR`, preserving relative paths (implies multiple files).                      |
| `--dry-run`                     | Only print what would be done, don’t write anything.                                            |
| `--diff`                        | Show unified diff instead of writing (implies `--dry-run`).                                     |
| `--threads <N>`                 | Number of parallel threads (default: number of CPU cores).                                      |
| `-v, --verbose`                 | Increase logging verbosity (`-v` for info, `-vv` for debug, `-vvv` for trace).                  |
| `-q, --quiet`                   | Suppress all output except errors.                                                              |
| `--json`                        | Output results as JSON (for integration).                                                       |
| `--config <FILE>`               | Load settings from a TOML file (see [Configuration](#configuration)).                           |
| `-f, --force`                   | Continue processing if some files fail.                                                         |
| `-h, --help`                    | Print help.                                                                                     |
| `-V, --version`                 | Print version.                                                                                  |

### Examples

#### Basic file processing

```bash
# Remove comments from a Rust file, print to stdout
rmcm main.rs

# Process multiple JavaScript files in‑place
rmcm -i *.js
```

#### Recursive directory

```bash
# Remove comments from all Python files in src/ (and subdirectories)
rmcm -r -i src/
```

#### Output directory

```bash
# Clean all C++ files in current directory, write results to ./cleaned/
rmcm --output-dir ./cleaned/ *.cpp
```

#### Stdin

```bash
# Read from stdin, specify language, print cleaned result
cat messy.py | rmcm -l python
```

#### Whitespace collapsing

```bash
# After removing comments, keep at most 1 blank line
rmcm -c 1 -i script.js
```

#### Diff mode (see changes without modifying)

```bash
rmcm --diff main.rs
```

#### Dry run

```bash
rmcm --dry-run -r src/
```

#### JSON output (for scripts/editors)

```bash
rmcm --json -r src/ > result.json
```

#### Force continue on errors

```bash
rmcm -f -i *.rs   # keep going even if some files fail
```

#### Custom thread count

```bash
rmcm --threads 2 -r src/   # use only 2 threads
```

---

## Configuration

You can store default options in a TOML file (e.g., `.rmcm.toml` in your project root) and pass it with `--config`.  
CLI flags **override** the values from the configuration file.

**Example `.rmcm.toml`**:

```toml
language = "rust"
collapse_whitespace = 1
recursive = true
output_dir = "cleaned"
threads = 4
```

Then run:

```bash
rmcm --config .rmcm.toml src/
```

All fields are optional. The available fields match the long option names:

- `language` – string
- `collapse_whitespace` – integer
- `recursive` – boolean
- `output_dir` – string (path)
- `threads` – integer

---

## Language Support

Each language is an **optional feature**. Enable only the ones you need to keep the binary small.  
The table below shows the feature flag, common file extensions, and the comment styles that are removed.

| Language   | Feature Flag | Extensions                                    | Comment Styles          |
| ---------- | ------------ | --------------------------------------------- | ----------------------- |
| Bash       | `bash`       | `.sh`, `.bash`                                | `#`                     |
| C          | `c`          | `.c`, `.h`                                    | `//`, `/* */`           |
| C#         | `c-sharp`    | `.cs`                                         | `//`, `/* */`, `///`    |
| C++        | `cpp`        | `.cpp`, `.cc`, `.cxx`, `.hpp`, `.hxx`, `.c++` | `//`, `/* */`           |
| CSS        | `css`        | `.css`                                        | `/* */`                 |
| Go         | `go`         | `.go`                                         | `//`, `/* */`           |
| Haskell    | `haskell`    | `.hs`                                         | `--`, `{- -}`           |
| HTML       | `html`       | `.html`, `.htm`                               | `<!-- -->`              |
| Java       | `java`       | `.java`                                       | `//`, `/* */`, `/** */` |
| JavaScript | `javascript` | `.js`, `.jsx`, `.mjs`, `.cjs`                 | `//`, `/* */`           |
| Lua        | `lua`        | `.lua`                                        | `--`, `--[[ ]]`         |
| PHP        | `php`        | `.php`                                        | `//`, `#`, `/* */`      |
| Python     | `python`     | `.py`, `.pyw`                                 | `#`                     |
| Ruby       | `ruby`       | `.rb`                                         | `#`, `=begin`/`=end`    |
| Rust       | `rust-lang`  | `.rs`                                         | `//`, `/* */`           |
| Scala      | `scala`      | `.scala`                                      | `//`, `/* */`           |
| Swift      | `swift`      | `.swift`                                      | `//`, `/* */`           |
| TypeScript | `typescript` | `.ts`, `.tsx`, `.mts`, `.cts`                 | `//`, `/* */`           |
| SQL        | `sql`        | `.sql`                                        | `--`, `/* */`           |
| Perl       | `perl`       | `.pl`, `.pm`, `.t`                            | `#`, POD                |
| R          | `r`          | `.r`, `.Rdata`                                | `#`                     |
| Dart       | `dart`       | `.dart`                                       | `//`, `/* */`           |
| Elixir     | `elixir`     | `.ex`, `.exs`                                 | `#`                     |
| TOML       | `toml`       | `.toml`                                       | `#`                     |
| INI        | `ini`        | `.ini`, `.cfg`, `.conf`                       | `;`, `#`                |

### Feature groups

For convenience, you can enable groups of related languages:

- `web` – `javascript`, `typescript`, `html`, `css`
- `web-backend` – `php`, `ruby`, `python`
- `jvm` – `java`, `scala`
- `systems` – `c`, `cpp`, `rust-lang`, `go`
- `scripting` – `bash`, `python`, `ruby`, `lua`
- `dotnet` – `c-sharp`
- `all` – everything listed above

Example installation with groups:

```bash
cargo install comment-remover --features "web,systems"
```

---

## Output Modes

The tool handles output in several ways:

1. **Single file to stdout** – default when exactly one file is given and neither `--in-place` nor `--output-dir` is used.
2. **In‑place** – `-i` overwrites the original file.
3. **Output directory** – `--output-dir DIR` writes files into `DIR`, preserving relative paths. Directories are created automatically.
4. **Diff / Dry‑run** – no files are written; a diff or log is shown.
5. **JSON** – when `--json` is used, the tool prints a summary (for multiple files) or the cleaned content wrapped in JSON (for stdin). The summary contains counts of successes and failures.

---

## Parallel Processing

By default, `rmcm` uses all available CPU cores to process files concurrently.  
You can control this with `--threads N`. The tool is I/O‑bound for many small files, but CPU‑bound for large files, so parallelism helps.

---

## Building from Source

```bash
git clone https://github.com/rhythmcache/comment-remover
cd comment-remover

# Build with default languages (C, C++, Rust, JavaScript, Python)
cargo build --release

# Build with all languages
cargo build --release --all-features

# Build with a custom set
cargo build --release --features "python,rust-lang,sql"
```

The binary will be at `target/release/rmcm`. You can copy it to a directory in your `PATH`.

---

## Contributing

Contributions are welcome! Whether it’s a bug report, a new language grammar, a feature request, or a pull request – feel free to open an [issue](https://github.com/rhythmcache/comment-remover/issues) or PR.

Some ideas for contributions:

- Add more tree‑sitter grammars (Kotlin, Julia, …)
- Improve comment queries for existing languages
- Add a `--preserve` flag to keep comments matching a pattern (e.g., license headers)
- Implement comment extraction (instead of removal)
- Add glob support for input files (currently the shell expands globs)

See the [open issues](https://github.com/rhythmcache/comment-remover/issues) for more.

For detailed information about the codebase and how to add a new language, please refer to the [CONTRIBUTING.md](CONTRIBUTING.md) file.

---

## Documentation

For more detailed documentation (including API docs and design notes), check out the [mdBook documentation](https://github.com/rhythmcache/comment-remover/tree/main/docs).  
You can build it locally with:

```bash
cd docs
mdbook build
open book/index.html   # or serve with `mdbook serve`
```

---

## License

This project is licensed under the Apache License, Version 2.0.  
See the [LICENSE](./license.md) file for details.