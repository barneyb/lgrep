use anyhow::{Context, Result};
use clap::{CommandFactory, Parser};

use cli::Cli;

use crate::handler::Handler;
use crate::Exit::Help;

mod cli;
mod handler;
mod io;

#[derive(Eq, PartialEq)]
pub enum Exit {
    Help,
    Error,
    Terminate,
    NoMatch,
    Match,
}

/// Run the grep, returning how many records matched.
pub fn run() -> Result<Exit> {
    let args = Cli::parse();
    // if no-filename (-h) without any patterns
    if args.no_filename && !args.has_patterns() {
        Cli::command()
            .print_help()
            .with_context(|| "failed to print help")?;
        Ok(Help)
    } else if args.help {
        Cli::command()
            .print_long_help()
            .with_context(|| "failed to print long help")?;
        Ok(Help)
    } else {
        let handler: Handler = args.into();
        handler.run()
    }
}
