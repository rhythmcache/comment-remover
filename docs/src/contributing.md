# Contributing to comment-remover

Thank you for considering contributing to **comment-remover**! We welcome all forms of contributions, including bug reports, feature requests, documentation improvements, and code changes.

## Table of Contents
- [Code of Conduct](#code-of-conduct)
- [Reporting Bugs](#reporting-bugs)
- [Suggesting Features](#suggesting-features)
- [Development Setup](#development-setup)
- [Adding Support for a New Language](#adding-support-for-a-new-language)
- [Coding Guidelines](#coding-guidelines)
- [Running Tests](#running-tests)
- [Pull Request Process](#pull-request-process)
- [License](#license)

## Code of Conduct
This project adheres to the [Contributor Covenant Code of Conduct](https://www.contributor-covenant.org/version/2/1/code_of_conduct/). By participating, you are expected to uphold this code. Please report unacceptable behavior to [rhythmcache](mailto:rhythmcache@example.com) (update with actual email).

## Reporting Bugs
If you find a bug, please open an issue on [GitHub Issues](https://github.com/rhythmcache/comment-remover/issues) and include:
- A clear, descriptive title.
- Steps to reproduce the bug.
- Expected behavior vs actual behavior.
- Version of comment-remover (`rmcm --version`).
- Your environment (OS, architecture).
- If possible, attach a minimal code snippet that triggers the bug.

## Suggesting Features
We welcome new ideas! Open an issue with the `enhancement` label and describe:
- The feature you'd like and why it's useful.
- Example usage (if applicable).
- Whether you're willing to implement it (we can guide you).

## Development Setup
1. **Fork** the repository to your GitHub account.
2. **Clone** your fork locally:
   ```bash
   git clone https://github.com/your-username/comment-remover.git
   cd comment-remover
   ```
3. Ensure you have Rust installed (edition 2024). Visit [rustup.rs](https://rustup.rs/) for installation.
4. Build the project:
   ```bash
   cargo build
   ```
5. Run the binary:
   ```bash
   cargo run -- --help
   ```

## Adding Support for a New Language
comment-remover uses **tree-sitter** parsers to remove comments accurately. To add a new language:

1. **Find a tree-sitter crate** for your language on [crates.io](https://crates.io/) (e.g., `tree-sitter-java`).
2. **Add the dependency** in `Cargo.toml` under `[dependencies]` as **optional**:
   ```toml
   tree-sitter-yourlang = { version = "...", optional = true }
   ```
3. **Add a feature flag** for the language in the `[features]` section:
   ```toml
   yourlang = ["dep:tree-sitter-yourlang"]
   ```
   If the language belongs to a group (e.g., `web`, `systems`), add it to the corresponding feature group.
4. **Implement the language parser** in the codebase:
   - Create a new module or extend an existing one (e.g., in `src/language/`).
   - Define a constant for the language using `tree_sitter_yourlang()`.
   - Implement the necessary traits (like `CommentParser`) if required.
   - Register the language in the language registry (typically in `src/language/mod.rs` or similar).
5. **Add tests**:
   - Create a sample file containing various comments for that language.
   - Add integration tests to verify comments are removed correctly.
6. **Update documentation**:
   - List the new language in the README.
   - Update CLI help text if needed.

## Coding Guidelines
- Follow the **Rust 2024 edition** conventions.
- Format your code with `rustfmt`:
  ```bash
  cargo fmt --all -- --check
  ```
- Lint your code with `clippy`:
  ```bash
  cargo clippy --all-features -- -D warnings
  ```
- Write clear comments for non‑obvious logic.
- Document public functions and types with `///`.
- Use `anyhow` for user‑friendly error reporting and `thiserror` for domain errors.

## Running Tests
The project includes unit tests and integration tests. Run all tests with:
```bash
cargo test --all-features
```
Make sure all tests pass before submitting a pull request.

## Pull Request Process
1. **Create a new branch** with a descriptive name:
   ```bash
   git checkout -b feature/your-feature
   # or
   git checkout -b fix/issue-description
   ```
2. **Make your changes**, committing with clear messages (prefer [Conventional Commits](https://www.conventionalcommits.org/)):
   ```
   feat: add support for language X
   fix: correct comment stripping in strings for language Y
   docs: update README with new languages
   ```
3. **Ensure code quality**:
   ```bash
   cargo fmt --all -- --check
   cargo clippy --all-features -- -D warnings
   cargo test --all-features
   ```
4. **Push your branch** to your fork:
   ```bash
   git push origin feature/your-feature
   ```
5. **Open a Pull Request** against the `main` branch of the original repository.
   - Provide a clear description of the changes.
   - Reference any related issues (e.g., "Closes #123").
6. **Wait for review**. We'll try to respond within a few days. Address any requested changes by pushing additional commits to the same branch.

## License
By contributing, you agree that your contributions will be licensed under the [Apache License 2.0](LICENSE).

---

Thank you for helping improve comment-remover! 🚀