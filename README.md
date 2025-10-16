# Comment Remover

Strip comments from your code.

## Install

```bash
cargo install --git "https://github.com/rhythmcache/comment-remover"
```

Usage :

```bash
rmcm -l shell < script.sh > clean.sh
rmcm -l python < script.py > clean.py
rmcm -l c < main.c > clean.c
rmcm -l basic < file.txt > clean.txt
```

Or pipe it directly:

```bash
cat script.sh | rmcm -l shell
```

## Supported Languages

- **Shell** (`shell`) - Bash, Zsh, sh with full heredoc support
- **Python** (`python`) - Python with docstring handling
- **C-style** (`c`) - C, C++, Java, JavaScript, Go, Rust, etc.
- **Basic** (`basic`) - Generic `#` comment removal

## TODO

Want to help? Here's what we're planning:

- [ ] **XML/HTML** - Handle `<!-- comments -->`
- [ ] **SQL** - Support `--` and `/* */` comments
- [ ] **Ruby** - Add `#` comment support with heredoc
- [ ] **Lua** - Handle `--` and `--[[ ]]` block comments
- [ ] **Lisp/Clojure** - Support `;` comments
- [ ] **Go** - Improve existing C-style support
- [ ] **YAML** - Handle `#` comments properly
- [ ] **LaTeX** - Support `%` comments
- [ ] **Vim Script** - Handle `"` comments
- [ ] **Configuration files** - `.ini`, `.conf`, `.env`, etc.

## Contributing

Found a bug? PRs are welcome. repo is open to anything. Pick something from the TODO or bring your own.

## License

Apache 2.0
