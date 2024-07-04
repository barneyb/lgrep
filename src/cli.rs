use clap::Parser;
use regex::Regex;

const DEFAULT_LOG_PATTERN: &str = r"^\d{4}-\d{2}-\d{2}[ T]\d{2}:\d{2}:\d{2}([.,]\d+)?";

#[derive(Parser, Debug)]
#[command(
    version,
    about,
    author,
    arg_required_else_help = true,
    disable_help_flag = true
)]
pub struct Cli {
    /// Pattern to search
    pub pattern: Option<Regex>,

    /// File(s) to search. If omitted or '-', search STDIN.
    #[arg(required = false, default_value = "-", hide_default_value = true)]
    pub files: Vec<String>, // todo

    /// Additional patterns to search
    #[arg(short = 'e', long = "regexp", value_name = "pattern")]
    pub patterns: Vec<Regex>,

    /// Perform case-insensitive matching. By default, lgrep is case-sensitive.
    #[arg(
        short,
        long,
        long_help = "Perform case-insensitive matching. By default, lgrep is case-sensitive. Note \
                     that this flag applies to ALL patterns, including the log and, if provided, \
                     the start/end patterns. If you need finer control, don't use this option, but \
                     instead enable case-insensitivity within the pattern via a '(?i)' prefix. Or \
                     turn it off (via '(?-i)') and on throughout the pattern. All of Rust's 'regex' \
                     crate's capabilities are available (see \
                     https://docs.rs/regex/latest/regex/#grouping-and-flags for the nitty-gritty)."
    )]
    pub ignore_case: bool, // todo

    /// Stop reading the file after num matches
    #[arg(short, long, value_name = "num")]
    pub max_count: Option<usize>, // todo

    /// Selected lines are those not matching any of the specified patterns
    #[arg(short = 'v', long)]
    pub invert_match: bool, // todo

    /// Pattern identifying the start of a log record.
    #[arg(
        long,
        required = false,
        default_value = DEFAULT_LOG_PATTERN,
        hide_default_value = true,
        value_name = "pattern",
        long_help = "Pattern identifying the start of a log record. By default, assumes log records \
                     start with an ISO-8601 datetime with either second or sub-second precision. \
                     The 'T' may be replaced with a space, fractional seconds may be delmited with \
                     a '.' (period) or a ',' (comma), and a timezone is not required."
    )]
    pub log_pattern: Regex,

    /// Ignore records until this pattern is found in a file.
    #[arg(
        short = 'S',
        long,
        value_name = "pattern",
        long_help = "Ignore records until this pattern is found in a file. The record containing \
                     the pattern will be searched, and if it matches, printed."
    )]
    pub start: Option<Regex>,

    /// Ignore remaining records once this pattern is found in a file.
    #[arg(
        short = 'E',
        long,
        value_name = "pattern",
        long_help = "Ignore remaining records once this pattern is found in a file. The record \
                     containing the pattern will not be searched."
    )]
    pub end: Option<Regex>,

    /// Always print filename headers with output lines.
    #[arg(short = 'H', long)]
    pub filename: bool, // todo

    /// Never print filename headers with output lines.
    #[arg(short = 'h', long)]
    pub no_filename: bool, // todo

    /// Print a brief help message.
    #[arg(long)]
    pub help: bool,
}

fn opt_re_match(opt_re: &Option<Regex>, hay: &str) -> bool {
    if let Some(re) = &opt_re {
        re.is_match(hay)
    } else {
        false
    }
}

impl Cli {
    pub fn has_patterns(&self) -> bool {
        self.pattern.is_some() || !self.patterns.is_empty()
    }

    pub fn is_match(&self, hay: &str) -> bool {
        opt_re_match(&self.pattern, hay) || self.patterns.iter().any(|re| re.is_match(hay))
    }

    pub fn has_start(&self) -> bool {
        self.start.is_some()
    }

    pub fn is_start(&self, hay: &str) -> bool {
        opt_re_match(&self.start, hay)
    }

    pub fn has_end(&self) -> bool {
        self.end.is_some()
    }

    pub fn is_end(&self, hay: &str) -> bool {
        opt_re_match(&self.end, hay)
    }

    pub fn is_record_start(&self, hay: &str) -> bool {
        self.log_pattern.is_match(hay)
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
                log_pattern: DEFAULT_LOG_PATTERN.parse().unwrap(),
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
        assert_eq!(false, cli.has_patterns());
    }

    #[test]
    fn implicit_pattern() {
        let cli = Cli {
            pattern: Some(".".parse().unwrap()),
            ..Cli::empty()
        };
        assert_eq!(true, cli.has_patterns());
    }

    #[test]
    fn explicit_patterns() {
        let cli = Cli {
            patterns: vec![".".parse().unwrap()],
            ..Cli::empty()
        };
        assert_eq!(true, cli.has_patterns());
    }

    #[test]
    fn no_start() {
        let cli = Cli::empty();
        assert_eq!(false, cli.has_start())
    }

    #[test]
    fn with_start() {
        let cli = Cli {
            start: Some(".".parse().unwrap()),
            ..Cli::empty()
        };
        assert_eq!(true, cli.has_start())
    }

    #[test]
    fn no_end() {
        let cli = Cli::empty();
        assert_eq!(false, cli.has_end())
    }

    #[test]
    fn with_end() {
        let cli = Cli {
            end: Some(".".parse().unwrap()),
            ..Cli::empty()
        };
        assert_eq!(true, cli.has_end())
    }

    #[test]
    fn no_match_no_patterns() {
        let cli = Cli::empty();
        assert_eq!(false, cli.has_patterns());
    }

    #[test]
    fn match_implicit_pattern() {
        let cli = Cli {
            pattern: Some("a".parse().unwrap()),
            ..Cli::empty()
        };
        assert_eq!(true, cli.is_match("bab"));
    }

    #[test]
    fn no_match_implicit_pattern() {
        let cli = Cli {
            pattern: Some("z".parse().unwrap()),
            ..Cli::empty()
        };
        assert_eq!(false, cli.is_match("bab"));
    }

    #[test]
    fn match_explicit_pattern() {
        let cli = Cli {
            patterns: vec!["a".parse().unwrap()],
            ..Cli::empty()
        };
        assert_eq!(true, cli.is_match("bab"));
    }

    #[test]
    fn match_explicit_patterns() {
        let cli = Cli {
            patterns: vec!["a".parse().unwrap(), "b".parse().unwrap()],
            ..Cli::empty()
        };
        assert_eq!(true, cli.is_match("bab"));
        assert_eq!(true, cli.is_match("xxxaxxx"));
        assert_eq!(true, cli.is_match("xxxbxxx"));
    }

    #[test]
    fn no_match_explicit_patterns() {
        let cli = Cli {
            patterns: vec!["y".parse().unwrap(), "z".parse().unwrap()],
            ..Cli::empty()
        };
        assert_eq!(false, cli.is_match("bab"));
    }

    #[test]
    fn no_match_no_start() {
        let cli = Cli::empty();
        assert_eq!(false, cli.is_start("bab"));
    }

    #[test]
    fn match_start() {
        let cli = Cli {
            start: Some("a".parse().unwrap()),
            ..Cli::empty()
        };
        assert_eq!(true, cli.is_start("bab"));
    }

    #[test]
    fn no_match_start() {
        let cli = Cli {
            pattern: Some("z".parse().unwrap()),
            ..Cli::empty()
        };
        assert_eq!(false, cli.is_start("bab"));
    }

    #[test]
    fn no_match_no_end() {
        let cli = Cli::empty();
        assert_eq!(false, cli.is_end("bab"));
    }

    #[test]
    fn match_end() {
        let cli = Cli {
            end: Some("a".parse().unwrap()),
            ..Cli::empty()
        };
        assert_eq!(true, cli.is_end("bab"));
    }

    #[test]
    fn no_match_end() {
        let cli = Cli {
            pattern: Some("z".parse().unwrap()),
            ..Cli::empty()
        };
        assert_eq!(false, cli.is_end("bab"));
    }

    #[test]
    fn record_start_default() {
        let cli = Cli::empty();
        assert_eq!(
            true,
            cli.is_record_start(
                "2024-07-01 01:25:47.755 Unexpected error occurred in scheduled task"
            )
        );
    }

    #[test]
    fn no_record_start_default() {
        let cli = Cli::empty();
        assert_eq!(false,  cli.is_record_start("    at org.springframework.orm.jpa.JpaTransactionManager.doBegin(JpaTransactionManager.java:466)"));
    }

    #[test]
    fn record_start_custom() {
        let cli = Cli {
            log_pattern: "GOAT".parse().unwrap(),
            ..Cli::empty()
        };
        assert_eq!(true, cli.is_record_start("i am a GOAT or something?"));
    }

    #[test]
    fn no_record_start_custom() {
        let cli = Cli {
            log_pattern: "GOAT".parse().unwrap(),
            ..Cli::empty()
        };
        assert_eq!(false, cli.is_record_start("definitely only a rabbit"));
    }

    #[test]
    fn case_sensitivity_or_not() {
        // todo
        let mut re = Regex::new("a").unwrap();
        assert_eq!(true, re.is_match("000a000"));
        assert_eq!(false, re.is_match("000A000"));
        re = Regex::new(&*(r"(?i)".to_owned() + re.as_str())).unwrap();
        assert_eq!(true, re.is_match("000a000"));
        assert_eq!(true, re.is_match("000A000"));
    }
}
