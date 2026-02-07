# Comment Remover

Strip comments from your code

## Install

## Quick install through cargo binstall
```bash
cargo binstall comment-remover
```
- or Build from source

```bash
cargo install comment-remover --all-features
```

### Install specific languages only
for small binary sizes, install only the languages you need:

```bash
# example: Install with Python and Rust support only
cargo install comment-remover --features "python,rust-lang"

# example: install with JavaScript, TypeScript, and C support
cargo install comment-remover --features "javascript,typescript,c"
```

## Usage

```
rmcm [OPTIONS] [FILES]...

Arguments:
  [FILES]...  Input files to process (if empty, reads from stdin)

Options:
  -l, --language <LANG>           Specify language (required for stdin, optional for files)
  -i, --in-place                  Modify files in-place instead of outputting to stdout
  -c, --collapse-whitespace <N>   Collapse consecutive blank lines to at most N blank lines
  -f, --force                     Continue processing even if some files fail
  -h, --help                      Print help
```

## Examples

```bash
# remove comments from a python file and print to stdout
rmcm script.py

# remove comments and modify file in-place
rmcm -i main.rs

# process multiple files in-place
rmcm -i *.js

# read from stdin (language must be specified)
cat script.py | rmcm -l python

# collapse multiple blank lines to at most 1
rmcm -c 1 -i code.js

# override language detection
rmcm -l python file.txt

# continue processing even if some files fail
rmcm -f -i *.rs
```

## Supported Languages

The tool uses tree-sitter parsers for accurate comment removal. Each language is an optional feature:

| Language | Feature Flag | Extensions | Comment Styles |
|----------|-------------|------------|----------------|
| **Bash** | `bash` | `.sh`, `.bash` | `#` |
| **C** | `c` | `.c`, `.h` | `//`, `/* */` |
| **C#** | `c-sharp` | `.cs` | `//`, `/* */`, `///` |
| **C++** | `cpp` | `.cpp`, `.cc`, `.cxx`, `.hpp`, `.hxx`, `.c++` | `//`, `/* */` |
| **CSS** | `css` | `.css` | `/* */` |
| **Go** | `go` | `.go` | `//`, `/* */` |
| **Haskell** | `haskell` | `.hs` | `--`, `{- -}` |
| **HTML** | `html` | `.html`, `.htm` | `<!-- -->` |
| **Java** | `java` | `.java` | `//`, `/* */`, `/** */` |
| **JavaScript** | `javascript` | `.js`, `.jsx`, `.mjs`, `.cjs` | `//`, `/* */` |
| **Lua** | `lua` | `.lua` | `--`, `--[[ ]]` |
| **PHP** | `php` | `.php` | `//`, `#`, `/* */` |
| **Python** | `python` | `.py`, `.pyw` | `#` |
| **Ruby** | `ruby` | `.rb` | `#`, `=begin`/`=end` |
| **Rust** | `rust-lang` | `.rs` | `//`, `/* */` |
| **Scala** | `scala` | `.scala` | `//`, `/* */` |
| **Swift** | `swift` | `.swift` | `//`, `/* */` |
| **TypeScript** | `typescript` | `.ts`, `.tsx`, `.mts`, `.cts` | `//`, `/* */` |

## Build from Source

```bash
# clone the repo
git clone https://github.com/rhythmcache/comment-remover
cd comment-remover

# build with all languages
cargo build --release --all-features

# Build with specific languages
cargo build --release --features "python,javascript,rust-lang"
```

## Available Features

All language features:
```
bash, c, c-sharp, cpp, css, go, haskell, html, java, javascript, 
json, lua, php, python, ruby, rust-lang, scala, swift, typescript, yaml
```

## TODO

Want to help? Here's what we're planning:
- [ ] **SQL** - Add support for SQL dialects with `--` and `/* */`
- [ ] **Perl** - Support `#` comments and POD documentation
- [ ] **R** - Handle `#` comments
- [ ] **Kotlin** - Support `//` and `/* */` comments
- [ ] **Dart** - Add comment support
- [ ] **Elixir** - Handle `#` comments
- [ ] **Configuration files** - `.ini`, `.conf`, `.env`, `.toml`
- [ ] **Documentation generation** - Option to extract comments instead of removing
- [ ] **Preserve specific comments** - Keep comments matching patterns (e.g., license headers)
- [ ] **Recursive directory processing** - Process entire directory trees
- [ ] **Parallel processing** - Speed up batch operations


## Contributing
Found a bug? PRs are welcome. The repo is open to anything. Pick something from the TODO or bring your own ideas.

## License
Apache 2.0
