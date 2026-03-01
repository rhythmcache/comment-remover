# Install Guide

This page explains how to install **comment-remover** (binary name `rmcm`) on your system.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Installation Methods](#installation-methods)
  - [Using cargo-binstall (quick)](#using-cargo-binstall-quick)
  - [Using cargo install](#using-cargo-install)
  - [Building from source](#building-from-source)
- [Feature Flags (Language Selection)](#feature-flags-language-selection)
- [Verifying the Installation](#verifying-the-installation)

---

## Prerequisites

- **Rust toolchain** (only if you build from source or use `cargo install`).  
  Install via [rustup](https://rustup.rs/) if you don't have it.
- **No runtime dependencies** – the binary is statically linked.

---

## Installation Methods

### Using cargo-binstall (quick)

If you have [cargo-binstall](https://github.com/cargo-bins/cargo-binstall) installed, you can download a pre‑compiled binary directly from GitHub releases:

```bash
cargo binstall comment-remover
```

This is the fastest method and does not require compiling the code.

### Using cargo install

You can install directly from [crates.io](https://crates.io/crates/comment-remover) using Cargo:

```bash
cargo install comment-remover
```

By default, this installs a **minimal set** of languages (C, C++, Rust, JavaScript, Python) – the `default` feature.  
To enable more languages, use the `--features` flag (see [Feature Flags](#feature-flags-language-selection) below).

After installation, the binary `rmcm` will be placed in `~/.cargo/bin/` (make sure this directory is in your `PATH`).

### Building from source

Clone the repository and build with Cargo:

```bash
git clone https://github.com/rhythmcache/comment-remover
cd comment-remover
cargo build --release
```

The binary will be at `target/release/rmcm`. You can optionally install it with:

```bash
cargo install --path .
```

or simply copy it to a directory in your `PATH`.

---

## Feature Flags (Language Selection)

`comment-remover` uses optional features for each programming language.  
This allows you to keep the binary small by including only the languages you need.

**Basic syntax:**

```bash
cargo install comment-remover --features "lang1,lang2"
```

or when building from source:

```bash
cargo build --release --features "lang1,lang2"
```

### Available language features

| Language   | Feature flag   |
|------------|----------------|
| Bash       | `bash`         |
| C          | `c`            |
| C#         | `c-sharp`      |
| C++        | `cpp`          |
| CSS        | `css`          |
| Go         | `go`           |
| Haskell    | `haskell`      |
| HTML       | `html`         |
| Java       | `java`         |
| JavaScript | `javascript`   |
| Lua        | `lua`          |
| PHP        | `php`          |
| Python     | `python`       |
| Ruby       | `ruby`         |
| Rust       | `rust-lang`    |
| Scala      | `scala`        |
| Swift      | `swift`        |
| TypeScript | `typescript`   |
| SQL        | `sql`          |
| Perl       | `perl`         |
| R          | `r`            |
| Dart       | `dart`         |
| Elixir     | `elixir`       |
| TOML       | `toml`         |
| INI        | `ini`          |

### Feature groups

You can also enable groups of languages:

- `web` – `javascript`, `typescript`, `html`, `css`
- `web-backend` – `php`, `ruby`, `python`
- `jvm` – `java`, `scala`
- `systems` – `c`, `cpp`, `rust-lang`, `go`
- `scripting` – `bash`, `python`, `ruby`, `lua`
- `dotnet` – `c-sharp`
- `all` – all languages listed above

**Example:**

```bash
cargo install comment-remover --features "web,systems"
```

This installs support for JavaScript, TypeScript, HTML, CSS, C, C++, Rust, and Go.

---

## Verifying the Installation

After installation, check that the binary works:

```bash
rmcm --version
```

You should see output like `comment-remover 0.2.1`.  
To see the list of available options:

```bash
rmcm --help
```

If you encounter any issues, ensure that `~/.cargo/bin` is in your `PATH`, or open an [issue](https://github.com/rhythmcache/comment-remover/issues).
