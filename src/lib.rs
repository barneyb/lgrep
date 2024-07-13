use std::process::ExitCode;

use anyhow::{Context, Result};
use clap::{CommandFactory, Parser};

use cli::Cli;

use crate::handler::Handler;
use crate::Exit::{Help, NoMatch};

mod cli;
mod handler;
mod io;

#[derive(Eq, PartialEq, Debug)]
pub enum Exit {
    Help,
    Error,
    Terminate,
    NoMatch,
    Match,
}

impl From<Exit> for ExitCode {
    fn from(value: Exit) -> Self {
        use Exit::*;
        // these match grep's behavior
        ExitCode::from(match value {
            Help => 2,
            Error => 2,
            NoMatch => 1,
            Match | Terminate => 0,
        })
    }
}

/// Run the grep, returning how many records matched.
pub fn run() -> Result<Exit> {
    let args = Cli::parse().like_grep();
    // if no-filename (-h) without any patterns
    if args.no_filename && !args.has_patterns() {
        Cli::command()
            .print_help()
            .context("failed to print help")?;
        Ok(Help)
    } else if args.help {
        Cli::command()
            .print_long_help()
            .context("failed to print long help")?;
        Ok(Help)
    } else if let Some(0) = args.max_count {
        // weird, but permitted
        Ok(NoMatch)
    } else {
        let handler: Handler = args.into();
        handler.run()
    }
}
