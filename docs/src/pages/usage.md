# Usage Guide

This page covers how to use `rmcm` effectively, with examples for common tasks.

## Table of Contents

- [Basic Usage](#basic-usage)
- [Command Line Options](#command-line-options)
- [Examples](#examples)
  - [Process a single file](#process-a-single-file)
  - [Process multiple files](#process-multiple-files)
  - [Recursive directory processing](#recursive-directory-processing)
  - [In‑place editing](#in-place-editing)
  - [Output directory](#output-directory)
  - [Stdin input](#stdin-input)
  - [Whitespace collapsing](#whitespace-collapsing)
  - [Diff mode](#diff-mode)
  - [Dry run](#dry-run)
  - [JSON output](#json-output)
  - [Force continue](#force-continue)
  - [Custom thread count](#custom-thread-count)
- [Output Modes Explained](#output-modes-explained)
- [Language Detection](#language-detection)

---

## Basic Usage

```bash
rmcm [OPTIONS] [FILES]...
```

If no files are given, the tool reads from standard input. In that case, you **must** specify the language with `--language` (or `-l`).

## Command Line Options

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
| `--config <FILE>`               | Load settings from a TOML file (see [Configuration](config.md)).                                |
| `-f, --force`                   | Continue processing if some files fail.                                                         |
| `-h, --help`                    | Print help.                                                                                     |
| `-V, --version`                 | Print version.                                                                                  |

---

## Examples

### Process a single file

```bash
rmcm main.rs
```
Removes comments from `main.rs` and prints the result to stdout.

### Process multiple files

```bash
rmcm file1.js file2.js file3.js
```
Processes all given files and prints each result to stdout (they will be concatenated). Usually you want to use `--in-place` or `--output-dir` for multiple files.

### Recursive directory processing

```bash
rmcm -r src/
```
Processes all files inside `src/` (and its subdirectories) that have a supported extension. Without `-i` or `--output-dir`, it would print all results to stdout (not recommended). Combine with `-i` or `--output-dir`.

### In‑place editing

```bash
rmcm -i script.py
```
Overwrites `script.py` with the comment‑free version.

```bash
rmcm -i -r src/
```
Recursively processes all files in `src/` and overwrites them in‑place.

### Output directory

```bash
rmcm --output-dir ./cleaned/ *.cpp
```
Processes all `.cpp` files in the current directory and writes the cleaned versions to `./cleaned/`, preserving the original filenames.

```bash
rmcm --output-dir ../stripped -r src/
```
Processes everything under `src/` and recreates the directory structure inside `../stripped`.

### Stdin input

```bash
cat messy.py | rmcm -l python
```
Reads from standard input, treats it as Python, and prints the cleaned code.

You can also combine with output redirection:

```bash
curl https://example.com/script.js | rmcm -l javascript > cleaned.js
```

### Whitespace collapsing

After comment removal, you may end up with many blank lines. Use `-c` to limit them.

```bash
rmcm -c 1 -i file.rs
```
Keeps at most **one** consecutive blank line. Multiple blank lines are reduced to a single blank line.

```bash
rmcm -c 0 -i file.py
```
Removes **all** blank lines (except those inside non‑comment code).

### Diff mode

Preview changes without modifying files:

```bash
rmcm --diff main.rs
```
Shows a unified diff of what would be removed.

### Dry run

See which files would be processed:

```bash
rmcm --dry-run -r src/
```
Logs each file that would be processed, but does not write any output.

### JSON output

For integration with tools or scripts, use `--json`:

```bash
rmcm --json -r src/ > result.json
```
Outputs a JSON object with `success`, `failed`, and `failures` fields.

For stdin, the JSON output contains the cleaned code:

```bash
cat code.py | rmcm -l python --json
```
```json
{
  "result": "print('hello')\n"
}
```

### Force continue

If some files cannot be processed (e.g., unsupported language, parse errors), the tool normally stops. Use `-f` to skip them and continue with the rest:

```bash
rmcm -f -i *.rs
```

### Custom thread count

Control parallelism:

```bash
rmcm --threads 2 -r src/
```
Uses only 2 threads instead of the default (all CPU cores).

---

## Output Modes Explained

`rmcm` has several output modes depending on the combination of flags:

| Mode                | Conditions                                                                 | Behaviour                                                                 |
|---------------------|----------------------------------------------------------------------------|---------------------------------------------------------------------------|
| **Stdout**          | Exactly one file, no `-i`, no `--output-dir`, no `--diff`, no `--dry-run` | Prints cleaned code to stdout.                                            |
| **In‑place**        | `-i` is set                                                                | Overwrites the original file(s) with cleaned content.                     |
| **Output directory**| `--output-dir` is set                                                      | Writes files into that directory, preserving relative paths.              |
| **Diff**            | `--diff` is set                                                            | Shows a diff for each file, does not write anything.                      |
| **Dry run**         | `--dry-run` is set (and `--diff` is not)                                   | Logs what would be done, no writes.                                       |
| **JSON**            | `--json` is set                                                            | Outputs a JSON summary (for multiple files) or the cleaned code (stdin). |

If multiple files are given and none of `-i`, `--output-dir`, `--diff`, or `--dry-run` are used, the tool will **error** because printing multiple files concatenated to stdout is ambiguous.

---

## Language Detection

By default, `rmcm` detects the language from the file extension (see the table in [Language Support](install.md#feature-flags-language-selection)).  
You can override detection with `-l` or `--language`. This is mandatory when reading from stdin.

If a file has an unknown extension or the language feature is not enabled, the tool will report an error and (unless `-f` is used) stop.
