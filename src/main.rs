//! Binary entry point for the comment‑remover tool.
//!
//! This file contains the `main` function, which parses command‑line arguments
//! using [`clap`] and runs the application. If an error occurs, it is printed
//! to stderr and the process exits with a non‑zero status code.
//!
//! The actual work is delegated to [`cli::Cli::run`].

use clap::Parser;
use comment_remover::cli::Cli;
use comment_remover::error::Result;
use std::process;

/// The main function.
///
/// Calls `run()` and handles any returned error by printing it to stderr
/// and exiting with status code 1.
fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {}", err);
        process::exit(1);
    }
}

/// The inner logic of the program.
///
/// Parses the command line (using [`Cli::parse`]) and executes the requested
/// operation. Returns a [`Result`] indicating success or failure.
fn run() -> Result<()> {
    let cli = Cli::parse();
    cli.run()
}
