use clap::Parser;
use regex::Regex;

#[derive(Parser, Debug)]
#[command(
    version,
    about,
    author,
    arg_required_else_help = true,
    disable_help_flag = true,
    after_long_help = "COMPRESSED LOGS:
\n\
                       Files (and STDIN) will be automatically decompressed, assuming appropriate \
                       utilities are available on your $PATH. That is, 'gzcat log.gz | lgrep ERROR' \
                       is unneeded; just do 'lgrep ERROR log.gz' (but don't do 'zlgrep ERROR log.gz').
\n\
                       ENVIRONMENT:
\n\
                       The LGREP_LOG_PATTERN environment variable may be used to default the \
                       '--log-pattern' option, if you consistently need a different start-of-record \
                       pattern in your environment. Providing the option supersedes the variable.
\n\
                       There is no support for a GREP_OPTIONS equivalent. Use a shell function."
)]
pub(crate) struct Cli {
    /// Pattern to search.
    #[arg(
        long_help = "Pattern to search. Like 'grep', if any PATTERN are passed with '-e', all \
                     positional params will be considered filenames."
    )]
    pub pattern: Option<String>,

    /// File(s) to search. If omitted or '-', search STDIN.
    pub files: Vec<String>,

    /// Additional patterns to search.
    #[arg(
        short = 'e',
        long = "regexp",
        value_name = "PATTERN",
        long_help = "Additional patterns to search. Unlike 'grep', a syntax error in PATTERN will \
                     exit with a helpful message and a non-zero exit code. An invalid positional \
                     PATTERN is ignored (like 'grep')."
    )]
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

    /// Only a count of selected records is written to standard output.
    #[arg(short, long)]
    pub count: bool,

    /// Label to use in place of “(standard input)” for a file name where a file name would normally be printed.
    #[arg(long)]
    pub label: Option<String>,

    /// Pattern identifying the start of a log record.
    #[arg(
        long,
        value_name = "PATTERN",
        long_help = "Pattern identifying the first line of a log record. By default, assumes log \
                     records start with an ISO-8601-ish datetime with sub-second precision. The 'T' \
                     may be replaced with a space, and fractional seconds may be delimited with a \
                     '.' (period) or a ',' (comma). Timezone is not required. To make lgrep behave \
                     like grep, pass '' (match everything) as the log pattern.
\n\
                     Before the first log record starts, each line is treated as a separate record, \
                     as if invoked as grep.
\n\
                     Be careful if you pipe a multi-file lgrep into another lgprep! By default, the \
                     second lgrep will receive filename-prefixed lines, which your log pattern must \
                     gracefully handle. The default pattern accounts for this."
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
}

#[cfg(test)]
mod tests {
    use clap::{CommandFactory, FromArgMatches};

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
                count: false,
                log_pattern: None,
                start: None,
                end: None,
                filename: false,
                no_filename: false,
                help: false,
            }
        }
    }

    impl From<&str> for Cli {
        fn from(value: &str) -> Self {
            Self::from(value.trim().split_whitespace().collect::<Vec<_>>())
        }
    }

    impl From<Vec<&str>> for Cli {
        fn from(value: Vec<&str>) -> Self {
            let mut matches = Cli::command().try_get_matches_from(value.iter()).unwrap();
            <Self as FromArgMatches>::from_arg_matches_mut(&mut matches).unwrap()
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

    // grep -e Cli - < src/handler.rs
    // grep -e Cli < src/handler.rs
    // grep Cli - < src/handler.rs
    // grep Cli < src/handler.rs
    // grep -e Cli -e H src/handler.rs
    // grep -e Cli H src/handler.rs
    // grep -e Cli src/*.rs
    // grep -e Cli src/handler.rs
    // grep Cli H src/handler.rs
    // grep Cli src/*.rs
    // grep Cli src/handler.rs

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
        let cli = Cli::from("lgrep -e Cli -").like_grep();
        assert_eq!(None, cli.pattern);
        assert_patterns(vec!["Cli"], &cli.patterns);
        assert!(cli.has_patterns());
        assert_files(vec!["-"], &cli.files);
    }

    #[test]
    fn like_grep_2() {
        let cli = Cli::from("lgrep -e Cli").like_grep();
        assert_eq!(None, cli.pattern);
        assert_patterns(vec!["Cli"], &cli.patterns);
        assert!(cli.has_patterns());
        assert_files(vec![], &cli.files);
    }

    #[test]
    fn like_grep_3() {
        let cli = Cli::from("lgrep Cli -").like_grep();
        assert_eq!(Some("Cli".to_owned()), cli.pattern);
        assert_patterns(vec![], &cli.patterns);
        assert!(cli.has_patterns());
        assert_files(vec!["-"], &cli.files);
    }

    #[test]
    fn like_grep_4() {
        let cli = Cli::from("lgrep Cli").like_grep();
        assert_eq!(Some("Cli".to_owned()), cli.pattern);
        assert_patterns(vec![], &cli.patterns);
        assert!(cli.has_patterns());
        assert_files(vec![], &cli.files);
    }

    #[test]
    fn like_grep_5() {
        let cli = Cli::from("lgrep -e Cli -e H src/handler.rs").like_grep();
        assert_eq!(None, cli.pattern);
        assert_patterns(vec!["Cli", "H"], &cli.patterns);
        assert!(cli.has_patterns());
        assert_files(vec!["src/handler.rs"], &cli.files);
    }

    #[test]
    fn like_grep_6_1() {
        let cli = Cli::from("lgrep -e Cli H src/handler.rs").like_grep();
        assert_eq!(None, cli.pattern);
        assert_patterns(vec!["Cli"], &cli.patterns);
        assert!(cli.has_patterns());
        assert_files(vec!["H", "src/handler.rs"], &cli.files);
    }

    #[test]
    fn like_grep_6_2() {
        let cli = Cli::from("lgrep H -e Cli src/handler.rs").like_grep();
        assert_eq!(None, cli.pattern);
        assert_patterns(vec!["Cli"], &cli.patterns);
        assert!(cli.has_patterns());
        assert_files(vec!["H", "src/handler.rs"], &cli.files);
    }

    #[test]
    fn like_grep_7() {
        let cli = Cli::from("lgrep -e Cli src/cli.rs src/handler.rs").like_grep();
        assert_eq!(None, cli.pattern);
        assert_patterns(vec!["Cli"], &cli.patterns);
        assert!(cli.has_patterns());
        assert_files(vec!["src/cli.rs", "src/handler.rs"], &cli.files);
    }

    #[test]
    fn like_grep_8() {
        let cli = Cli::from("lgrep -e Cli src/handler.rs").like_grep();
        assert_eq!(None, cli.pattern);
        assert_patterns(vec!["Cli"], &cli.patterns);
        assert!(cli.has_patterns());
        assert_files(vec!["src/handler.rs"], &cli.files);
    }

    #[test]
    fn like_grep_9() {
        let cli = Cli::from("lgrep Cli H src/handler.rs").like_grep();
        assert_eq!(Some("Cli".to_owned()), cli.pattern);
        assert_patterns(vec![], &cli.patterns);
        assert!(cli.has_patterns());
        assert_files(vec!["H", "src/handler.rs"], &cli.files);
    }

    #[test]
    fn like_grep_10() {
        let cli = Cli::from("lgrep Cli src/cli.rs src/handler.rs").like_grep();
        assert_eq!(Some("Cli".to_owned()), cli.pattern);
        assert_patterns(vec![], &cli.patterns);
        assert!(cli.has_patterns());
        assert_files(vec!["src/cli.rs", "src/handler.rs"], &cli.files);
    }

    #[test]
    fn like_grep_11() {
        let cli = Cli::from("lgrep Cli src/handler.rs").like_grep();
        assert_eq!(Some("Cli".to_owned()), cli.pattern);
        assert_patterns(vec![], &cli.patterns);
        assert!(cli.has_patterns());
        assert_files(vec!["src/handler.rs"], &cli.files);
    }
}
