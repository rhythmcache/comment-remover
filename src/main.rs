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
