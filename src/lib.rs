use std::process::ExitCode;

use anyhow::Result;
use clap::Parser;

use cli::Cli;

use crate::handler::Handler;

mod cli;
mod handler;
mod read;
mod write;

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

impl From<usize> for Exit {
    fn from(match_count: usize) -> Self {
        if match_count == 0 {
            Exit::NoMatch
        } else {
            Exit::Match
        }
    }
}

/// Run the grep, returning how many records matched.
pub fn run() -> Result<Exit> {
    let args = Cli::parse().like_grep();
    // if no-filename (-h) without any patterns
    if args.no_filename && !args.has_patterns() {
        args.print_help()
    } else if args.help {
        args.print_long_help()
    } else if let Some(0) = args.max_count {
        // weird, but permitted
        Ok(Exit::NoMatch)
    } else {
        let handler = Handler::new(args)?;
        handler.run()
    }
}
