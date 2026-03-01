
# Configuration Guide

You can store default options in a TOML configuration file and pass it with `--config`.  
This is useful for projects where you always want the same behaviour (e.g., recursive, output directory, whitespace collapsing).

## Table of Contents

- [Configuration File Format](#configuration-file-format)
- [Example](#example)
- [Merging with CLI Arguments](#merging-with-cli-arguments)
- [Where to Place the File](#where-to-place-the-file)

---

## Configuration File Format

The configuration file is a simple TOML document. All fields are optional.  
The available fields correspond to the long option names:

| Field                 | Type    | Description                                      |
|-----------------------|---------|--------------------------------------------------|
| `language`            | string  | Default language (overrides auto‑detection).     |
| `collapse_whitespace` | integer | Maximum consecutive blank lines (`-c`).          |
| `recursive`           | boolean | Process directories recursively (`-r`).          |
| `output_dir`          | string  | Directory for output (`--output-dir`).           |
| `threads`             | integer | Number of threads (`--threads`).                  |

**Note:** Boolean flags like `in_place`, `dry_run`, `diff`, `json`, `force` are **not** stored in the config file because they are typically one‑off decisions. You must provide them on the command line.

## Example

Create a file named `.rmcm.toml` in your project root:

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

This is equivalent to:

```bash
rmcm -l rust -c 1 -r --output-dir cleaned --threads 4 src/
```

## Merging with CLI Arguments

Command‑line arguments **override** values from the configuration file.  
For example, if your config sets `recursive = true`, but you run:

```bash
rmcm --config .rmcm.toml --no-recursive src/
```

(Note: there is no `--no-recursive` flag; you simply omit `-r`. In practice, CLI flags take precedence: if you don't pass `-r`, the config's `recursive = true` applies; if you do pass `-r`, it applies. But `-r` is a flag, so you cannot "unset" it.)

For boolean flags like `recursive`, the behaviour is:  
- If `-r` is present on the command line → recursive = true  
- Else → use the value from config (if present) or default (false).

For options with values (`language`, `collapse_whitespace`, `output_dir`, `threads`), the CLI value always wins if provided; otherwise the config value is used.

## Where to Place the File

You can put the configuration file anywhere, but it is common to place it in the project root (e.g., `.rmcm.toml`). You must explicitly pass its path with `--config`; the tool does **not** automatically search for a config file.

If you find yourself always using the same configuration, consider creating an alias in your shell:

```bash
alias rmcm='rmcm --config ~/.rmcm.toml'
```

But remember that CLI flags will still override the config.