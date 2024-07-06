use anyhow::{Context, Result};
use clap::{CommandFactory, Parser};

use cli::Cli;

use crate::handler::Handler;

mod cli;
mod handler;
mod io;

/// Run the grep, returning how many records matched.
pub fn run() -> Result<isize> {
    let args = Cli::parse();
    // if no-filename (-h) without any patterns
    if args.no_filename && !args.has_patterns() {
        Cli::command()
            .print_help()
            .with_context(|| "failed to print help")?;
        Ok(-1)
    } else if args.help {
        Cli::command()
            .print_long_help()
            .with_context(|| "failed to print long help")?;
        Ok(-1)
    } else {
        let handler: Handler = args.into();
        match handler.run() {
            Ok(n) => Ok(n as isize),
            Err(e) => Err(e),
        }
    }
}
