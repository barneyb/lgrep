use anyhow::{Context, Result};
use clap::{ColorChoice, CommandFactory, Parser};
use regex::Regex;

use crate::Exit;
use crate::Exit::Help;

#[cfg(not(target_os = "windows"))]
const COMPRESSED_FILES: &str = "COMPRESSED FILES:
\n\
                       Files (and STDIN) will be automatically decompressed, assuming appropriate \
                       utilities are available on your `$PATH`. That is, `gzcat log.gz | lgrep ERROR` \
                       is unneeded; just do `lgrep ERROR log.gz` (but don't do `zlgrep ERROR log.gz`). \
                       This feature is not available on Windows.
\n\
                       ";

const BASE_LONG_HELP: &str = "ENVIRONMENT:
\n\
                       The `LGREP_LOG_PATTERN` environment variable may be used to default the \
                       '--log-pattern' option, if you consistently need a different start-of-record \
                       pattern in your environment. Providing the option supersedes the variable.
\n\
                       The `GREP_COLORS` environment variable will be used to color output, in \
                       similar manner as `grep`. All `grep` capabilities are accepted, but not all \
                       affect output. For example, `lgrep` doesn't have context lines.
\n\
                       There is no support for a `GREP_OPTIONS` equivalent. Use a shell function.";

#[derive(Debug, Parser)]
#[command(
    version,
    about,
    author,
    arg_required_else_help = true,
    disable_help_flag = true,
    after_long_help = BASE_LONG_HELP,
)]
pub(crate) struct Cli {
    /// Pattern to search.
    ///
    /// Like `grep`, if any PATTERN are passed with '-e', all positional params
    /// are considered filenames.
    #[arg()]
    pub pattern: Option<String>,

    /// File(s) to search. If omitted or '-', search STDIN.
    pub files: Vec<String>,

    /// Additional patterns to search.
    ///
    /// Unlike `grep`, a syntax error in PATTERN will exit with a helpful message and a non-zero
    /// exit code. An invalid positional PATTERN is ignored (like `grep`).
    #[arg(short = 'e', long = "regexp", value_name = "PATTERN")]
    pub patterns: Vec<Regex>,

    /// Perform case-insensitive matching.
    ///
    /// By default, `lgrep` is case-sensitive. Note that this flag applies to ALL patterns,
    /// including the log and, if provided, the start/end patterns. If you need finer control,
    /// enable case-insensitivity within the pattern via a `(?i)` prefix (which is what this option
    /// does internally). Or turn it off (via `(?-i)`) and on throughout the pattern. All of Rust's
    /// 'regex' crate's capabilities are available (see
    /// https://docs.rs/regex/latest/regex/#grouping-and-flags for the nitty-gritty).
    #[arg(short, long)]
    pub ignore_case: bool,

    /// Stop reading the file after num matches.
    #[arg(short, long, value_name = "NUM")]
    pub max_count: Option<usize>,

    /// Selected lines are those NOT matching any of the specified patterns.
    ///
    /// Does not impact log/start/end patterns, only the main matching pattern(s).
    #[arg(short = 'v', long)]
    pub invert_match: bool,

    /// Only a count of selected records is written to standard output.
    #[arg(short, long)]
    pub count: bool,

    /// Label to use in place of “(standard input)” for a file name where a file name would normally
    /// be printed.
    #[arg(long)]
    pub label: Option<String>,

    /// Color output, according to a subset of `GREP_COLOR`.
    ///
    /// Surround the matched (non-empty) strings and file names with escape sequences to display
    /// them in color on the terminal. The colors are defined by the environment variable
    /// `GREP_COLORS`.
    #[arg(
        long,
        visible_alias = "colour",
        value_name = "WHEN",
        default_value = "auto",
        default_missing_value = "always",
        num_args = 0..=1,
        require_equals = true,
    )]
    pub color: ColorChoice,

    /// Pattern identifying the start of a log record.
    ///
    /// By default, assumes log records start with an ISO-8601-ish datetime with sub-second
    /// precision. The 'T' may be replaced with a space, and fractional seconds may be delimited
    /// with a '.' (period) or a ',' (comma). Timezone is not required. To make `lgrep` behave like
    /// `grep`, pass '' (match everything) as the log pattern.
    ///
    /// Before the first log record starts, each line is treated as a separate record, as if invoked
    /// as `grep`.
    ////
    /// Be careful if you pipe a multi-file `lgrep` into another `lgrep`! By default, the second
    /// `lgrep` will receive filename-prefixed lines, which your log pattern must gracefully handle.
    /// The default pattern accounts for this.
    #[arg(long, value_name = "PATTERN", long_help = "")]
    pub log_pattern: Option<Regex>,

    /// Ignore records until this pattern is found in a file.
    ///
    /// The record containing the pattern WILL be searched, and if it matches, printed.
    #[arg(short = 'S', long, value_name = "PATTERN")]
    pub start: Option<Regex>,

    /// Ignore remaining records once this pattern is found in a file.
    ///
    /// The record containing the pattern WILL NOT be searched.
    #[arg(short = 'E', long, value_name = "PATTERN")]
    pub end: Option<Regex>,

    /// Always print filename headers with output lines.
    ///
    /// The first line of a record will follow the filename with a ':' (colon) and subsequent lines
    /// with a '-' (hyphen). This is reminiscent of `grep`'s contextual line formatting (via '-C').
    #[arg(short = 'H', long)]
    pub filename: bool,

    /// Never print filename headers with output lines. If '-H' is also specified, filenames aren't
    /// included.
    #[arg(short = 'h', long)]
    pub no_filename: bool,

    /// Print comprehensive help.
    #[arg(long)]
    pub help: bool,
}

impl Cli {
    pub(crate) fn like_grep(mut self) -> Self {
        if !self.patterns.is_empty() {
            if let Some(p) = self.pattern {
                // p is a file, since there are explict patterns
                self.pattern = None;
                self.files.insert(0, p);
            }
        }
        self
    }

    pub fn has_patterns(&self) -> bool {
        self.pattern.is_some() || !self.patterns.is_empty()
    }

    pub(crate) fn print_help(&self) -> Result<Exit> {
        Cli::command()
            .print_help()
            .context("failed to print help")?;
        Ok(Help)
    }

    #[cfg(not(target_os = "windows"))]
    pub(crate) fn print_long_help(&self) -> Result<Exit> {
        Cli::command_for_update()
            .after_long_help(COMPRESSED_FILES.to_owned() + BASE_LONG_HELP)
            .print_long_help()
            .context("failed to print long help")?;
        Ok(Help)
    }

    #[cfg(target_os = "windows")]
    pub(crate) fn print_long_help(&self) -> Result<Exit> {
        Cli::command()
            .print_long_help()
            .context("failed to print long help")?;
        Ok(Help)
    }
}

#[cfg(test)]
impl Cli {
    pub(crate) fn empty() -> Cli {
        Cli {
            pattern: None,
            files: vec![],
            patterns: vec![],
            ignore_case: false,
            max_count: None,
            invert_match: false,
            count: false,
            label: None,
            color: ColorChoice::Auto,
            log_pattern: None,
            start: None,
            end: None,
            filename: false,
            no_filename: false,
            help: false,
        }
    }

    pub(crate) fn all_re() -> Cli {
        Cli {
            pattern: Some(r"P".to_owned()),
            patterns: vec![r"Q".parse().unwrap(), r"R".parse().unwrap()],
            log_pattern: Some(r"L".parse().unwrap()),
            start: Some(r"S".parse().unwrap()),
            end: Some(r"E".parse().unwrap()),
            ..Self::empty()
        }
    }
}

#[cfg(test)]
mod tests {
    use clap::{CommandFactory, FromArgMatches};

    use super::*;

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

    #[cfg(test)]
    mod like_grep {
        use super::*;

        impl From<&str> for Cli {
            fn from(value: &str) -> Self {
                Self::from(value.trim().split_whitespace().collect::<Vec<_>>()).like_grep()
            }
        }

        impl From<Vec<&str>> for Cli {
            fn from(value: Vec<&str>) -> Self {
                let mut matches = Cli::command().try_get_matches_from(value.iter()).unwrap();
                <Self as FromArgMatches>::from_arg_matches_mut(&mut matches).unwrap()
            }
        }

        fn assert_patterns(left: Vec<&str>, right: &Vec<Regex>) {
            assert_eq!(
                left,
                right.iter().map(|p| p.to_string()).collect::<Vec<_>>()
            );
        }

        fn assert_files(left: Vec<&str>, right: &Vec<String>) {
            assert_eq!(left, right.iter().collect::<Vec<_>>());
        }

        #[test]
        fn like_grep_1() {
            let cli = Cli::from("lgrep -e Cli -");
            assert_eq!(None, cli.pattern);
            assert_patterns(vec!["Cli"], &cli.patterns);
            assert!(cli.has_patterns());
            assert_files(vec!["-"], &cli.files);
        }

        #[test]
        fn like_grep_2() {
            let cli = Cli::from("lgrep -e Cli");
            assert_eq!(None, cli.pattern);
            assert_patterns(vec!["Cli"], &cli.patterns);
            assert!(cli.has_patterns());
            assert_files(vec![], &cli.files);
        }

        #[test]
        fn like_grep_3() {
            let cli = Cli::from("lgrep Cli -");
            assert_eq!(Some("Cli".to_owned()), cli.pattern);
            assert_patterns(vec![], &cli.patterns);
            assert!(cli.has_patterns());
            assert_files(vec!["-"], &cli.files);
        }

        #[test]
        fn like_grep_4() {
            let cli = Cli::from("lgrep Cli");
            assert_eq!(Some("Cli".to_owned()), cli.pattern);
            assert_patterns(vec![], &cli.patterns);
            assert!(cli.has_patterns());
            assert_files(vec![], &cli.files);
        }

        #[test]
        fn like_grep_5() {
            let cli = Cli::from("lgrep -e Cli -e H src/handler.rs");
            assert_eq!(None, cli.pattern);
            assert_patterns(vec!["Cli", "H"], &cli.patterns);
            assert!(cli.has_patterns());
            assert_files(vec!["src/handler.rs"], &cli.files);
        }

        #[test]
        fn like_grep_6_1() {
            let cli = Cli::from("lgrep -e Cli H src/handler.rs");
            assert_eq!(None, cli.pattern);
            assert_patterns(vec!["Cli"], &cli.patterns);
            assert!(cli.has_patterns());
            assert_files(vec!["H", "src/handler.rs"], &cli.files);
        }

        #[test]
        fn like_grep_6_2() {
            let cli = Cli::from("lgrep H -e Cli src/handler.rs");
            assert_eq!(None, cli.pattern);
            assert_patterns(vec!["Cli"], &cli.patterns);
            assert!(cli.has_patterns());
            assert_files(vec!["H", "src/handler.rs"], &cli.files);
        }

        #[test]
        fn like_grep_7() {
            let cli = Cli::from("lgrep -e Cli src/cli.rs src/handler.rs");
            assert_eq!(None, cli.pattern);
            assert_patterns(vec!["Cli"], &cli.patterns);
            assert!(cli.has_patterns());
            assert_files(vec!["src/cli.rs", "src/handler.rs"], &cli.files);
        }

        #[test]
        fn like_grep_8() {
            let cli = Cli::from("lgrep -e Cli src/handler.rs");
            assert_eq!(None, cli.pattern);
            assert_patterns(vec!["Cli"], &cli.patterns);
            assert!(cli.has_patterns());
            assert_files(vec!["src/handler.rs"], &cli.files);
        }

        #[test]
        fn like_grep_9() {
            let cli = Cli::from("lgrep Cli H src/handler.rs");
            assert_eq!(Some("Cli".to_owned()), cli.pattern);
            assert_patterns(vec![], &cli.patterns);
            assert!(cli.has_patterns());
            assert_files(vec!["H", "src/handler.rs"], &cli.files);
        }

        #[test]
        fn like_grep_10() {
            let cli = Cli::from("lgrep Cli src/cli.rs src/handler.rs");
            assert_eq!(Some("Cli".to_owned()), cli.pattern);
            assert_patterns(vec![], &cli.patterns);
            assert!(cli.has_patterns());
            assert_files(vec!["src/cli.rs", "src/handler.rs"], &cli.files);
        }

        #[test]
        fn like_grep_11() {
            let cli = Cli::from("lgrep Cli src/handler.rs");
            assert_eq!(Some("Cli".to_owned()), cli.pattern);
            assert_patterns(vec![], &cli.patterns);
            assert!(cli.has_patterns());
            assert_files(vec!["src/handler.rs"], &cli.files);
        }
    }
}
