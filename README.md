# Comment Remover

A fast, accurate command-line tool to strip comments from source code using [tree‑sitter](https://tree-sitter.github.io/).  
It supports **dozens of languages**, works on single files or whole directories, and can run in parallel for maximum speed.

[![Crates.io](https://img.shields.io/crates/v/comment-remover.svg)](https://crates.io/crates/comment-remover)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)

---

## Features

- [x] **Accurate comment removal** – uses tree‑sitter’s precise syntax trees, so comments inside strings or literals are never touched.
- [x] **40+ languages** – from C, Rust, Python to SQL, TOML, and even INI files. Enable only what you need.
- [x] **Parallel processing** – automatically uses all CPU cores (configurable).
- [x] **Recursive directory traversal** – process entire source trees with `-r`.
- [x] **In‑place editing** – overwrite files directly with `-i`.
- [x] **Output directory** – write cleaned files to a separate folder, preserving the relative structure.
- [x] **Whitespace collapsing** – reduce consecutive blank lines to a desired maximum with `-c`.
- [x] **Diff mode** – see what would change without modifying files (`--diff`).
- [x] **Dry‑run** – preview which files would be processed.
- [ ] **JSON output** – integrate with editors or build tools.
- [x] **Configuration file** – store default options in `.rmcm.toml`.
- [ ] **Force continue** – keep going even if some files fail (`-f`).

---

## Installation

### Quick install with `cargo binstall`

If you have [cargo-binstall](https://github.com/cargo-bins/cargo-binstall):

```bash
cargo binstall comment-remover
```

### Install from source with `cargo`

```bash
cargo install comment-remover
```

By default this installs a subset of languages (C, C++, Rust, JavaScript, Python).  
To get **all** languages, use `--all-features`:

```bash
cargo install comment-remover --all-features
```

Or select exactly the languages you need (smaller binary):

```bash
# Example: Python and Rust only
cargo install comment-remover --features "python,rust-lang"

# Example: JavaScript, TypeScript, C, and C++
cargo install comment-remover --features "javascript,typescript,c,cpp"
```

> All available feature flags are listed in the [Language Support](#language-support) section.

### Build from the repository

```bash
git clone https://github.com/rhythmcache/comment-remover
cd comment-remover
cargo build --release
# binary is at target/release/rmcm
```

---

## Usage

The binary is named **`rmcm`** (short for “remove comments”).

```bash
rmcm [OPTIONS] [FILES]...
```

If no files are given, the tool reads from standard input (you **must** provide `--language` in that case).

### Options

| Option                          | Description                                                                    |
| ------------------------------- | ------------------------------------------------------------------------------ |
| `-l, --language <LANG>`         | Force language (overrides auto‑detection). Required for stdin.                 |
| `-i, --in-place`                | Edit files in‑place (overwrite original).                                      |
| `-c, --collapse-whitespace <N>` | Keep at most `N` consecutive blank lines. Use `0` to remove all.               |
| `-r, --recursive`               | Process directories recursively.                                               |
| `--output-dir <DIR>`            | Write output to `DIR`, preserving relative paths (implies multiple files).     |
| `--dry-run`                     | Only print what would be done, don’t write anything.                           |
| `--diff`                        | Show unified diff instead of writing (implies `--dry-run`).                    |
| `--threads <N>`                 | Number of parallel threads (default: number of CPU cores).                     |
| `-v, --verbose`                 | Increase logging verbosity (`-v` for info, `-vv` for debug, `-vvv` for trace). |
| `-q, --quiet`                   | Suppress all output except errors.                                             |
| `--json`                        | Output results as JSON (for integration).                                      |
| `--config <FILE>`               | Load settings from a TOML file (see [Configuration](#configuration)).          |
| `-f, --force`                   | Continue processing if some files fail.                                        |
| `-h, --help`                    | Print help.                                                                    |
| `-V, --version`                 | Print version.                                                                 |

---

## Examples

### Basic file processing

```bash
# Remove comments from a Rust file, print to stdout
rmcm main.rs

# Process multiple JavaScript files in‑place
rmcm -i *.js
```

### Recursive directory

```bash
# Remove comments from all Python files in src/ (and subdirectories)
rmcm -r -i src/
```

### Output directory

```bash
# Clean all C++ files in current directory, write results to ./cleaned/
rmcm --output-dir ./cleaned/ *.cpp
```

### Stdin

```bash
# Read from stdin, specify language, print cleaned result
cat messy.py | rmcm -l python
```

### Whitespace collapsing

```bash
# After removing comments, keep at most 1 blank line
rmcm -c 1 -i script.js
```

### Diff mode (see changes without modifying)

```bash
rmcm --diff main.rs
```

### Dry run

```bash
rmcm --dry-run -r src/
```

### JSON output (for scripts/editors)

```bash
rmcm --json -r src/ > result.json
```

### Force continue on errors

```bash
rmcm -f -i *.rs   # keep going even if some files fail
```

### Custom thread count

```bash
rmcm --threads 2 -r src/   # use only 2 threads
```

---

## Configuration

You can store default options in a TOML file (e.g., `.rmcm.toml` in your project root).  
Specify it with `--config` or let the tool look for it? Currently you must pass `--config`.

**Example `.rmcm.toml`**:

```toml
language = "rust"
collapse_whitespace = 1
recursive = true
output_dir = "cleaned"
threads = 4
```

CLI flags **override** the configuration file.

---

## Output Modes

The tool handles output in several ways:

1. **Single file to stdout** – default when exactly one file is given and neither `--in-place` nor `--output-dir` is used.
2. **In‑place** – `-i` overwrites the original file.
3. **Output directory** – `--output-dir DIR` writes files into `DIR`, preserving relative paths.
4. **Diff / Dry‑run** – no files are written; a diff or log is shown.
5. **JSON** – when `--json` is used, the tool prints a summary (for multiple files) or the cleaned content wrapped in JSON (for stdin). The summary contains counts of successes and failures.

---

## Language Support

Each language is an **optional feature** – enable only the ones you need to keep the binary small.  
The table shows the feature flag, common file extensions, and the comment styles that are removed.

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

Example:

```bash
cargo install comment-remover --features "web,systems"
```

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

The binary will be at `target/release/rmcm`.

---

## Contributing

Contributions are welcome! Whether it’s a bug report, a new language grammar, a feature request, or a pull request – feel free to open an issue or PR.

Some ideas for contributions:

- [ ] Add more tree‑sitter grammars (Kotlin, Julia, …)
- [ ] Improve comment queries for existing languages
- [ ] Add a `--preserve` flag to keep comments matching a pattern (e.g., license headers)
- [ ] Implement comment extraction (instead of removal)
- [ ] Add glob support for input files (currently the shell expands globs)

See the [open issues](https://github.com/rhythmcache/comment-remover/issues) for more.

---

## License

This project is licensed under the Apache License, Version 2.0.  
See [LICENSE](LICENSE) for details.
