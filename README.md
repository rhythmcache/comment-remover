# Comment Remover

Strip comments from your code.

## Install

```bash
cargo install --git "https://github.com/rhythmcache/comment-remover"
```

Usage :
```
rmcm [OPTIONS] [FILES]...

Arguments:
  [FILES]...

Options:
  -l, --language <LANG>
  -i, --in-place
  -c, --collapse-whitespace <N>
  -f, --force
  -h, --help                     Print help
```

## Supported Languages

- **Shell** (`shell`) - Bash, Zsh, sh with full heredoc support
- **Python** (`python`) - Python with docstring handling
- **C-style** (`c`) - C, C++, Java, JavaScript, Go, Rust, etc.
- **Basic** (`basic`) - Generic `#` comment removal

## TODO

Want to help? Here's what we're planning:

- [x] **XML/HTML** - Handle `<!-- comments -->`
- [ ] **SQL** - Support `--` and `/* */` comments
- [ ] **Ruby** - Add `#` comment support with heredoc
- [ ] **Lua** - Handle `--` and `--[[ ]]` block comments
- [ ] **Lisp/Clojure** - Support `;` comments
- [x] **Go** - Improve existing C-style support
- [ ] **YAML** - Handle `#` comments properly
- [ ] **LaTeX** - Support `%` comments
- [ ] **Vim Script** - Handle `"` comments
- [ ] **Configuration files** - `.ini`, `.conf`, `.env`, etc.

## Contributing

Found a bug? PRs are welcome. repo is open to anything. Pick something from the TODO or bring your own.

## License

Apache 2.0
