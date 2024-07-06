use clap::Parser;
use regex::Regex;

#[derive(Parser, Debug)]
#[command(
    version,
    about,
    author,
    arg_required_else_help = true,
    disable_help_flag = true
)]
pub(crate) struct Cli {
    /// Pattern to search
    pub pattern: Option<Regex>,

    /// File(s) to search. If omitted or '-', search STDIN.
    #[arg(required = false)]
    pub files: Vec<String>,

    /// Additional patterns to search
    #[arg(short = 'e', long = "regexp", value_name = "PATTERN")]
    pub patterns: Vec<Regex>,

    /// Perform case-insensitive matching.
    #[arg(
        short,
        long,
        long_help = "Perform case-insensitive matching. By default, lgrep is case-sensitive. Note \
                     that this flag applies to ALL patterns, including the log and, if provided, \
                     the start/end patterns. If you need finer control, enable case-insensitivity \
                     within the pattern via a '(?i)' prefix (which is what this option does \
                     internally). Or turn it off (via '(?-i)') and on throughout the pattern. All \
                     of Rust's 'regex' crate's capabilities are available (see \
                     https://docs.rs/regex/latest/regex/#grouping-and-flags for the nitty-gritty)."
    )]
    pub ignore_case: bool,

    /// Stop reading the file after num matches.
    #[arg(short, long, value_name = "NUM")]
    pub max_count: Option<usize>,

    /// Selected lines are those NOT matching any of the specified patterns
    #[arg(
        short = 'v',
        long,
        long_help = "Selected lines are those NOT matching any of the specified patterns. Does not \
                     impact log/start/end patterns, only the main matching pattern(s)."
    )]
    pub invert_match: bool,

    /// Label to use in place of “(standard input)” for a file name where a file name would normally be printed.
    #[arg(long)]
    pub label: Option<String>,

    /// Pattern identifying the start of a log record.
    #[arg(
        long,
        value_name = "PATTERN",
        long_help = "Pattern identifying the start of a log record. By default, assumes log records \
                     start with an ISO-8601 datetime with either second or sub-second precision. \
                     The 'T' may be replaced with a space, fractional seconds may be delimited with \
                     a '.' (period) or a ',' (comma), and a timezone is not required. To make lgrep \
                     behave like a slower grep, pass '' (match everything) as the log pattern."
    )]
    pub log_pattern: Option<Regex>,

    /// Ignore records until this pattern is found in a file.
    #[arg(
        short = 'S',
        long,
        value_name = "PATTERN",
        long_help = "Ignore records until this pattern is found in a file. The record containing \
                     the pattern will be searched, and if it matches, printed."
    )]
    pub start: Option<Regex>,

    /// Ignore remaining records once this pattern is found in a file.
    #[arg(
        short = 'E',
        long,
        value_name = "PATTERN",
        long_help = "Ignore remaining records once this pattern is found in a file. The record \
                     containing the pattern will not be searched."
    )]
    pub end: Option<Regex>,

    /// Always print filename headers with output lines.
    #[arg(
        short = 'H',
        long,
        long_help = "Always print filename headers with output lines. The first line of a record \
                     will follow the filename with a ':' (colon) and subsequent lines with a '-' \
                     (hyphen). This is reminiscent of grep's contextual line formatting (via '-C')."
    )]
    pub filename: bool,

    /// Never print filename headers with output lines. Trumps '-H' if both are specified.
    #[arg(short = 'h', long)]
    pub no_filename: bool,

    /// Print comprehensive help.
    #[arg(long)]
    pub help: bool,
}

impl Cli {
    pub fn has_patterns(&self) -> bool {
        self.pattern.is_some() || !self.patterns.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Cli {
        fn empty() -> Cli {
            Cli {
                pattern: None,
                files: vec![],
                patterns: vec![],
                ignore_case: false,
                max_count: None,
                invert_match: false,
                label: None,
                log_pattern: None,
                start: None,
                end: None,
                filename: false,
                no_filename: false,
                help: false,
            }
        }
    }

    #[test]
    fn no_patterns() {
        let cli = Cli::empty();
        assert!(!cli.has_patterns());
    }

    #[test]
    fn implicit_pattern() {
        let cli = Cli {
            pattern: Some(".".parse().unwrap()),
            ..Cli::empty()
        };
        assert!(cli.has_patterns());
    }

    #[test]
    fn explicit_patterns() {
        let cli = Cli {
            patterns: vec![".".parse().unwrap()],
            ..Cli::empty()
        };
        assert!(cli.has_patterns());
    }

    #[test]
    fn no_match_no_patterns() {
        let cli = Cli::empty();
        assert!(!cli.has_patterns());
    }
}
