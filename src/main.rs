//! Entry point for the comment remover binary.
//!
//! This module contains the `main` function which parses command-line arguments
//! using [`clap`] and delegates to the [`cli`] module's [`Cli::run`] method.
//! If an error occurs, it prints the error to stderr and exits with code 1.
//!
//! # Usage
//!
//! The binary accepts the same arguments as documented in [`cli::Cli`]. Run
//! `comment-remover --help` for details.
//!
//! # Exit codes
//!
//! * `0` – Successful execution (or no files processed with `--force`).
//! * `1` – An error occurred (I/O, parsing, unsupported language, etc.).
//!
//! # Examples
//!
//! ```bash
//! comment-remover file.rs --in-place
//! comment-remover src/ --recursive --output-dir out/
//! cat script.py | comment-remover -l python
//! ```

use clap::Parser;
use comment_remover::cli::Cli;
use comment_remover::error::Result;
use std::process;

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {}", err);
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    cli.run()
}