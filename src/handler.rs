use std::io::{BufRead, Write};

use anyhow::{Context, Result};
use regex::{Regex, RegexSet};

use crate::cli::Cli;
use crate::io;
use crate::io::STD_IN_FILENAME;

const DEFAULT_LOG_PATTERN: &str = r"^\d{4}-\d{2}-\d{2}[ T]\d{2}:\d{2}:\d{2}([.,]\d+)?";
const DEFAULT_LABEL: &str = "(standard input)";
const INSENSITIVE_PREFIX: &str = "(?i)";

pub struct Handler {
    pub files: Vec<String>,
    pub pattern: RegexSet,
    pub max_count: Option<usize>,
    pub invert_match: bool,
    pub label: String,        // todo
    pub log_pattern: Regex,   // todo
    pub start: Option<Regex>, // todo
    pub end: Option<Regex>,   // todo
    pub filename: bool,       // todo
}

fn opt_re_match(opt_re: &Option<Regex>, hay: &str) -> bool {
    if let Some(re) = &opt_re {
        re.is_match(hay)
    } else {
        false
    }
}

impl Handler {
    pub(crate) fn run(&self) -> Result<usize> {
        let mut sink = std::io::stdout().lock();
        let mut match_count = 0;
        for f in self.files.iter() {
            let source = io::get_reader(f)?;
            self.process_file(source, &mut sink, &mut match_count)?;
            if self.is_max_reached(match_count) {
                break;
            }
        }
        Ok(match_count)
    }

    fn process_file(
        &self,
        mut source: Box<dyn BufRead>,
        sink: &mut dyn Write,
        match_count: &mut usize,
    ) -> Result<()> {
        let mut s = String::new();
        while let Ok(n) = source.read_line(&mut s) {
            if n == 0 {
                // reached EOF
                break;
            }
            if self.is_match(&s) {
                sink.write_all(s.as_bytes())?;
                *match_count += 1;
                if self.is_max_reached(*match_count) {
                    break;
                }
            }
            s.clear(); // todo: reduce capacity as well, if large?
        }
        Ok(())
    }

    fn is_max_reached(&self, match_count: usize) -> bool {
        if let Some(mc) = self.max_count {
            match_count >= mc
        } else {
            false
        }
    }

    pub(crate) fn is_match(&self, hay: &str) -> bool {
        self.invert_match ^ self.pattern.is_match(hay)
    }

    pub(crate) fn has_start(&self) -> bool {
        self.start.is_some()
    }

    pub(crate) fn is_start(&self, hay: &str) -> bool {
        opt_re_match(&self.start, hay)
    }

    pub(crate) fn has_end(&self) -> bool {
        self.end.is_some()
    }

    pub(crate) fn is_end(&self, hay: &str) -> bool {
        opt_re_match(&self.end, hay)
    }

    pub(crate) fn is_record_start(&self, hay: &str) -> bool {
        self.log_pattern.is_match(hay)
    }
}

fn insensitive_str(re: &str) -> String {
    INSENSITIVE_PREFIX.to_owned() + re
}

fn insensitive_re(re: Regex) -> Regex {
    Regex::new(&insensitive_str(re.as_str())).unwrap()
}

fn opt_insensitive_re(opt_re: Option<Regex>) -> Option<Regex> {
    opt_re.map(insensitive_re)
}

impl From<Cli> for Handler {
    fn from(cli: Cli) -> Self {
        let mut patterns = cli.patterns;
        if let Some(p) = cli.pattern {
            patterns.push(p);
        }
        let mut pattern_strings = patterns.iter().map(|p| p.to_string()).collect::<Vec<_>>();
        let mut log_pattern = cli
            .log_pattern
            .unwrap_or_else(|| DEFAULT_LOG_PATTERN.parse().unwrap());
        let mut start = cli.start;
        let mut end = cli.end;
        if cli.ignore_case {
            pattern_strings = pattern_strings.iter().map(|s| insensitive_str(s)).collect();
            log_pattern = insensitive_re(log_pattern);
            start = opt_insensitive_re(start);
            end = opt_insensitive_re(end);
        }
        let mut files = cli.files;
        if files.is_empty() {
            files.push(STD_IN_FILENAME.to_owned())
        }
        // no-filename wins, otherwise if requested or multi-file
        let filename = if cli.no_filename {
            false
        } else {
            cli.filename || files.len() > 1
        };
        Handler {
            pattern: RegexSet::new(&pattern_strings).unwrap(),
            filename,
            files,
            max_count: cli.max_count,
            label: cli.label.unwrap_or_else(|| DEFAULT_LABEL.to_owned()),
            invert_match: cli.invert_match,
            log_pattern,
            start,
            end,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    impl Handler {
        fn empty() -> Handler {
            Handler {
                files: Vec::new(),
                pattern: RegexSet::new(&[r"a"]).unwrap(),
                max_count: None,
                invert_match: false,
                label: DEFAULT_LABEL.to_owned(),
                log_pattern: DEFAULT_LOG_PATTERN.parse().unwrap(),
                start: None,
                end: None,
                filename: false,
            }
        }

        fn all_re() -> Handler {
            Handler {
                pattern: RegexSet::new(&[r"P", r"Q", r"R"]).unwrap(),
                log_pattern: r"T".parse().unwrap(),
                start: Some(r"S".parse().unwrap()),
                end: Some(r"E".parse().unwrap()),
                ..Self::empty()
            }
        }

        fn process_file_for_count(
            &self,
            mut source: Box<dyn BufRead>,
            sink: &mut dyn Write,
        ) -> Result<usize> {
            let mut match_count = 0;
            self.process_file(source, sink, &mut match_count)?;
            Ok(match_count)
        }
    }

    #[test]
    fn no_start() {
        let h = Handler::empty();
        assert!(!h.has_start());
    }

    #[test]
    fn with_start() {
        let h = Handler::all_re();
        assert!(h.has_start());
    }

    #[test]
    fn no_end() {
        let h = Handler::empty();
        assert!(!h.has_end());
    }

    #[test]
    fn with_end() {
        let h = Handler::all_re();
        assert!(h.has_end());
    }

    #[test]
    fn is_match() {
        let h = Handler::all_re();
        assert!(h.is_match("0P0"));
        assert!(!h.is_match("zzz"));
        assert!(h.is_match("0Q0"));
        assert!(!h.is_match("zzz"));
        assert!(h.is_match("0R0"));
        assert!(!h.is_match("zzz"));
    }

    #[test]
    fn is_record_start() {
        let h = Handler::all_re();
        assert!(h.is_record_start("0T0"));
        assert!(!h.is_record_start("zzz"));
    }

    #[test]
    fn is_record_start_default() {
        let h = Handler::empty();
        assert!(h.is_record_start(
            "2024-07-01 01:25:47.755 Unexpected error occurred in scheduled task"
        ));
        assert!(!h.is_record_start("    at org.springframework.orm.jpa.JpaTransactionManager.doBegin(JpaTransactionManager.java:466)"));
    }

    #[test]
    fn is_record_start_custom() {
        let h = Handler {
            log_pattern: "GOAT".parse().unwrap(),
            ..Handler::empty()
        };
        assert!(h.is_record_start("i am a GOAT or something?"));
        assert!(!h.is_record_start("definitely only a rabbit"));
    }

    #[test]
    fn is_start_none() {
        let h = Handler::empty();
        assert!(!h.is_start("0S0"));
    }

    fn is_start() {
        let h = Handler::all_re();
        assert!(h.is_start("0S0"));
        assert!(!h.is_start("zzz"));
    }

    #[test]
    fn is_end_none() {
        let h = Handler::empty();
        assert!(!h.is_end("0E0"));
    }

    #[test]
    fn is_end() {
        let h = Handler::all_re();
        assert!(h.is_end("0E0"));
        assert!(!h.is_end("zzz"));
    }

    #[test]
    fn simple_process_file() {
        let source = Box::new(Cursor::new(
            b"line one
line two
third line
line 4
",
        ));
        let handler = Handler {
            pattern: RegexSet::new(&[r"t"]).unwrap(),
            ..Handler::empty()
        };
        let mut sink = Cursor::new(Vec::new());
        let match_count = handler.process_file_for_count(source, &mut sink).unwrap();
        let bytes = sink.into_inner();
        assert_eq!(
            "line two
third line
",
            String::from_utf8(bytes).unwrap()
        );
        assert_eq!(2, match_count);
    }
}
