use anyhow::{Context, Result};
use clap::{CommandFactory, Parser};

use cli::Cli;

mod cli;

pub fn run() -> Result<()> {
    let args = Cli::parse().init();
    // if no-filename (-h) without any patterns
    if args.no_filename && !args.has_patterns() {
        Cli::command()
            .print_help()
            .with_context(|| "failed to print help")?
    } else if args.help {
        Cli::command()
            .print_long_help()
            .with_context(|| "failed to print long help")?
    } else {
        dbg!(args);
    }
    Ok(())
}
