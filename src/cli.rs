use clap::Parser;
use regex::Regex;

const DEFAULT_LOG_PATTERN: &str = r"^\d{4}-\d{2}-\d{2}[ T]\d{2}:\d{2}:\d{2}([.,]\d+)?";
const DEFAULT_LABEL: &str = "(standard input)";
pub const STD_IN_FILENAME: &str = "-";

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
    #[arg(required = false, default_value = STD_IN_FILENAME, hide_default_value = true)]
    pub files: Vec<String>, // todo

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

    /// Stop reading the file after num matches
    #[arg(short, long, value_name = "NUM")]
    pub max_count: Option<usize>, // todo

    /// Selected lines are those NOT matching any of the specified patterns
    #[arg(
        short = 'v',
        long,
        long_help = "Selected lines are those NOT matching any of the specified patterns. Does not \
                     impact log/start/end patterns, only the main matching pattern(s)."
    )]
    pub invert_match: bool,

    /// Label to use in place of “(standard input)” for a file name where a file name would normally be printed.
    #[arg(long, default_value = DEFAULT_LABEL, hide_default_value = true)]
    pub label: String, // todo

    /// Pattern identifying the start of a log record.
    #[arg(
        long,
        default_value = DEFAULT_LOG_PATTERN,
        hide_default_value = true,
        value_name = "PATTERN",
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
    #[arg(short = 'H', long)]
    pub filename: bool, // todo

    /// Never print filename headers with output lines.
    #[arg(short = 'h', long)]
    pub no_filename: bool, // todo

    /// Print comprehensive help.
    #[arg(long)]
    pub help: bool,
}

fn insensitive_re(re: Regex) -> Regex {
    Regex::new(&*("(?i)".to_owned() + re.as_str())).unwrap()
}

fn opt_insensitive_re(opt_re: Option<Regex>) -> Option<Regex> {
    if let Some(re) = opt_re {
        Some(insensitive_re(re))
    } else {
        opt_re
    }
}

impl Cli {
    pub(crate) fn init(self) -> Cli {
        if !self.ignore_case {
            return self;
        }
        Cli {
            ignore_case: false,
            pattern: opt_insensitive_re(self.pattern),
            patterns: self.patterns.into_iter().map(insensitive_re).collect(),
            log_pattern: insensitive_re(self.log_pattern),
            start: opt_insensitive_re(self.start),
            end: opt_insensitive_re(self.end),
            ..self
        }
    }
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
        self.invert_match
            ^ (opt_re_match(&self.pattern, hay) || self.patterns.iter().any(|re| re.is_match(hay)))
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

    pub(crate) fn empty() -> Cli {
        Cli {
            pattern: None,
            files: vec![],
            patterns: vec![],
            ignore_case: false,
            max_count: None,
            invert_match: false,
            label: DEFAULT_LABEL.to_owned(),
            log_pattern: DEFAULT_LOG_PATTERN.parse().unwrap(),
            start: None,
            end: None,
            filename: false,
            no_filename: false,
            help: false,
        }
    }

    pub(crate) fn all_re() -> Cli {
        Cli {
            pattern: Some("P".parse().unwrap()),
            patterns: vec!["Q".parse().unwrap(), "R".parse().unwrap()],
            log_pattern: "T".parse().unwrap(),
            start: Some("S".parse().unwrap()),
            end: Some("E".parse().unwrap()),
            ..Cli::empty()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_empty() {
        let cli = Cli::all_re().init();
        // pattern
        assert_eq!(false, cli.is_match("0p0"));
        assert_eq!(true, cli.is_match("0P0"));
        // patterns
        assert_eq!(false, cli.is_match("0q0"));
        assert_eq!(true, cli.is_match("0Q0"));
        assert_eq!(false, cli.is_match("0r0"));
        assert_eq!(true, cli.is_match("0R0"));
        // log_pattern
        assert_eq!(false, cli.is_record_start("0t0"));
        assert_eq!(true, cli.is_record_start("0T0"));
        // start
        assert_eq!(false, cli.is_start("0s0"));
        assert_eq!(true, cli.is_start("0S0"));
        // end
        assert_eq!(false, cli.is_end("0e0"));
        assert_eq!(true, cli.is_end("0E0"));
    }

    #[test]
    fn init_everything() {
        let cli = Cli {
            ignore_case: true,
            ..Cli::all_re()
        }
        .init();
        // pattern
        assert_eq!(true, cli.is_match("0p0"));
        assert_eq!(true, cli.is_match("0P0"));
        // patterns
        assert_eq!(true, cli.is_match("0q0"));
        assert_eq!(true, cli.is_match("0Q0"));
        assert_eq!(true, cli.is_match("0r0"));
        assert_eq!(true, cli.is_match("0R0"));
        // log_pattern
        assert_eq!(true, cli.is_record_start("0t0"));
        assert_eq!(true, cli.is_record_start("0T0"));
        // start
        assert_eq!(true, cli.is_start("0s0"));
        assert_eq!(true, cli.is_start("0S0"));
        // end
        assert_eq!(true, cli.is_end("0e0"));
        assert_eq!(true, cli.is_end("0E0"));
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
        assert_eq!(false, cli.is_match("bkb"));
    }

    #[test]
    fn match_implicit_pattern_invert() {
        let cli = Cli {
            pattern: Some("a".parse().unwrap()),
            invert_match: true,
            ..Cli::empty()
        };
        assert_eq!(false, cli.is_match("bab"));
        assert_eq!(true, cli.is_match("bkb"));
    }

    #[test]
    fn no_match_implicit_pattern() {
        let cli = Cli {
            pattern: Some("z".parse().unwrap()),
            ..Cli::empty()
        };
        assert_eq!(false, cli.is_match("bab"));
        assert_eq!(false, cli.is_match("bkb"));
    }

    #[test]
    fn match_explicit_pattern() {
        let cli = Cli {
            patterns: vec!["a".parse().unwrap()],
            ..Cli::empty()
        };
        assert_eq!(true, cli.is_match("bab"));
        assert_eq!(false, cli.is_match("BAB"));
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
        assert_eq!(false, cli.is_match("BAB"));
        assert_eq!(false, cli.is_match("XXXAXXX"));
        assert_eq!(false, cli.is_match("XXXBXXX"));
    }

    #[test]
    fn match_explicit_pattern_insensitive() {
        let cli = Cli {
            patterns: vec!["a".parse().unwrap()],
            ignore_case: true,
            ..Cli::empty()
        }
        .init();
        assert_eq!(true, cli.is_match("bab"));
        assert_eq!(true, cli.is_match("BAB"));
    }

    #[test]
    fn match_explicit_patterns_insensitive() {
        let cli = Cli {
            patterns: vec!["a".parse().unwrap(), "b".parse().unwrap()],
            ignore_case: true,
            ..Cli::empty()
        }
        .init();
        assert_eq!(true, cli.is_match("bab"));
        assert_eq!(true, cli.is_match("xxxaxxx"));
        assert_eq!(true, cli.is_match("xxxbxxx"));
        assert_eq!(true, cli.is_match("BAB"));
        assert_eq!(true, cli.is_match("XXXAXXX"));
        assert_eq!(true, cli.is_match("XXXBXXX"));
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
        assert_eq!(false, cli.is_record_start("definitely only a rabbit"));
    }
}
