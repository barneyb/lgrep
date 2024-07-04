use anyhow::{Context, Result};
use clap::{CommandFactory, Parser};

#[derive(Parser, Debug)]
#[command(
    version,
    about,
    author,
    arg_required_else_help = true,
    disable_help_flag = true
)]
struct Cli {
    /// Pattern to search
    pattern: Option<String>,

    /// File(s) to search. If omitted or '-', search STDIN.
    #[arg(required = false, default_value = "-", hide_default_value = true)]
    file: Vec<String>,

    /// Additional patterns to search
    #[arg(short = 'e', long = "regexp", value_name = "pattern")]
    patterns: Vec<String>,

    /// Perform case-insensitive matching. By default, lgrep is case-sensitive.
    #[arg(short, long)]
    ignore_case: bool,

    /// Stop reading the file after num matches
    #[arg(short, long)]
    max_count: Option<usize>,

    /// Selected lines are those not matching any of the specified patterns
    #[arg(short = 'v', long)]
    invert_match: bool,

    /// Pattern identifying the start of a log record.
    #[arg(
        long,
        required = false,
        default_value = r"^\d{4}-\d{2}-\d{2}[ T]\d{2}:\d{2}:\d{2}([.,]\d{3})?",
        hide_default_value = true,
        value_name = "pattern",
        long_help = "Pattern identifying the start of a log record. By default, assumes log records \
                     start with an ISO-8601 timestamp with either second or millisecond precision. \
                     The 'T' may be replaced with a space, fractional seconds may be delmited with \
                     a '.' (period) or a ',' (comma), and a timezone is not required."
    )]
    log_pattern: String,

    /// Ignore records until this pattern is found in a file.
    #[arg(
        short = 'S',
        long,
        value_name = "pattern",
        long_help = "Ignore records until this pattern is found in a file. The record containing \
                     the pattern will be searched, and if it matches, printed."
    )]
    start: Option<String>,

    /// Ignore remaining records once this pattern is found in a file.
    #[arg(
        short = 'E',
        long,
        value_name = "pattern",
        long_help = "Ignore remaining records once this pattern is found in a file. The record \
                     containing the pattern will not be searched."
    )]
    end: Option<String>,

    /// Always print filename headers with output lines.
    #[arg(short = 'H', long)]
    filename: bool,

    /// Never print filename headers with output lines.
    #[arg(short = 'h', long)]
    no_filename: bool,

    /// Print a brief help message.
    #[arg(long)]
    help: bool,
}

fn main() -> Result<()> {
    let args = Cli::parse();

    // if no-filename (-h) without any patterns
    if args.no_filename && args.pattern.is_none() && args.patterns.is_empty() {
        Cli::command()
            .print_help()
            .with_context(|| "failed to print help")?
    } else if args.help {
        Cli::command()
            .print_long_help()
            .with_context(|| "failed to print help")?
    } else {
        dbg!(args);
    }
    Ok(())
}
