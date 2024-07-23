use std::env;
use std::io::{BufWriter, ErrorKind, Write};

use anyhow::{Context, Error, Result};
use clap::ColorChoice;
use regex::{Regex, RegexSet};

use read::STDIN_FILENAME;

use crate::cli::Cli;
use crate::read::Source;
use crate::{read, Exit};

const ENV_LOG_PATTERN: &str = "LGREP_LOG_PATTERN";

const DEFAULT_LOG_PATTERN: &str = r"(^|:)\d{4}-\d{2}-\d{2}[ T]\d{2}:\d{2}:\d{2}[.,]\d";
const DEFAULT_STDIN_LABEL: &str = "(standard input)";

const INSENSITIVE_PREFIX: &str = "(?i)";

pub(crate) struct Handler {
    files: Vec<String>,
    pattern_set: RegexSet,
    max_count: Option<usize>,
    invert_match: bool,
    counts: bool,
    color_mode: ColorChoice,
    stdin_label: Option<String>,
    log_pattern: Regex,
    start: Option<Regex>,
    end: Option<Regex>,
    filenames: bool,
}

fn opt_re_match(opt_re: &Option<Regex>, hay: &str) -> bool {
    if let Some(re) = &opt_re {
        re.is_match(hay)
    } else {
        false
    }
}

type Sink = BufWriter<dyn Write>;

impl Handler {
    pub(crate) fn run(&self) -> Result<Exit> {
        let mut sink = BufWriter::new(std::io::stdout().lock());
        let mut exit = Exit::NoMatch;
        for f in self.files.iter() {
            let reader = read::get_reader(f)?;
            let source = Source::new(self.display_name_for_filename(f), reader);
            match self.process_file(source, &mut sink)? {
                Exit::Terminate => {
                    return Ok(Exit::Terminate);
                }
                Exit::Match => exit = Exit::Match,
                _ => {}
            }
        }
        Ok(exit)
    }

    fn display_name_for_filename<'a>(&'a self, f: &'a str) -> &'a str {
        if f == STDIN_FILENAME {
            if let Some(lbl) = &self.stdin_label {
                lbl
            } else {
                DEFAULT_STDIN_LABEL
            }
        } else {
            f
        }
    }

    fn process_file(&self, source: Source, sink: &mut Sink) -> Result<Exit> {
        let mut file_started = !self.has_start();
        let mut before_first_record = true;
        let mut match_count = 0;
        let filename = source.filename;
        let mut process_record = |record: &str| {
            if record.is_empty() {
                return Ok(true);
            }
            if self.is_end(&record) {
                return Ok(false); // reached end
            }
            if !file_started && self.is_start(&record) {
                file_started = true;
            }
            if file_started && self.is_match(&record) {
                if !self.counts {
                    self.write(sink, &record, filename)?;
                }
                match_count += 1;
                if self.is_max_reached(match_count) {
                    return Ok(false); // reached max count
                }
            }
            Ok::<bool, anyhow::Error>(true)
        };
        // an entire log record
        let mut record = String::new();
        for line in source.lines() {
            // while let soaks up an Err; we want to propagate it
            match line {
                Err(e) => {
                    return Err(e)
                        .with_context(|| format!("Failed to read line from '{}'", filename))
                }
                Ok(l) => {
                    let start_of_record = self.is_record_start(&l.text);
                    if before_first_record && start_of_record {
                        before_first_record = false;
                    }
                    if before_first_record || start_of_record {
                        if !process_record(&record)? {
                            record.clear(); // don't re-process post-loop
                            break;
                        }
                        // start a new record with line
                        record.clone_from(&l.text);
                    } else {
                        // add line to the current record
                        record.push_str(&l.text);
                    }
                }
            }
        }
        process_record(&record)?;
        if self.counts {
            self.write(sink, &format!("{match_count}\n"), filename)?;
        }
        Ok(if match_count == 0 {
            Exit::NoMatch
        } else {
            Exit::Match
        })
    }

    fn is_max_reached(&self, match_count: usize) -> bool {
        if let Some(mc) = self.max_count {
            match_count >= mc
        } else {
            false
        }
    }

    fn is_match(&self, hay: &str) -> bool {
        self.invert_match ^ self.pattern_set.is_match(hay)
    }

    fn has_start(&self) -> bool {
        self.start.is_some()
    }

    fn is_start(&self, hay: &str) -> bool {
        opt_re_match(&self.start, hay)
    }

    #[allow(dead_code)]
    fn has_end(&self) -> bool {
        self.end.is_some()
    }

    fn is_end(&self, hay: &str) -> bool {
        opt_re_match(&self.end, hay)
    }

    fn is_record_start(&self, hay: &str) -> bool {
        self.log_pattern.is_match(hay)
    }

    fn write(&self, sink: &mut Sink, record: &str, filename: &str) -> Result<Exit> {
        let r = if self.filenames {
            with_filename(sink, record, filename)
        } else {
            without_filename(sink, record)
        }
        .and_then(|_| sink.flush());
        if let Err(e) = r {
            return if e.kind() == ErrorKind::BrokenPipe {
                // nothing is listening anymore
                Ok(Exit::Terminate)
            } else {
                Err(Error::from(e)).context("Failed to write")
            };
        }
        Ok(Exit::Match)
    }
}

fn default_log_pattern() -> Regex {
    if let Ok(p) = env::var(ENV_LOG_PATTERN) {
        p.parse()
    } else {
        DEFAULT_LOG_PATTERN.parse()
    }
    .unwrap()
}

fn without_filename(sink: &mut Sink, record: &str) -> std::io::Result<()> {
    sink.write_all(record.as_bytes())
}

const DELIM_START: &[u8] = &[b':'];
const DELIM_FOLLOW: &[u8] = &[b'-'];

fn with_filename(sink: &mut Sink, record: &str, filename: &str) -> std::io::Result<()> {
    let fn_bytes = filename.as_bytes();
    let lines = record.as_bytes().split_inclusive(|b| *b == b'\n');
    let mut delim = DELIM_START;
    for l in lines {
        sink.write_all(fn_bytes)?;
        sink.write_all(delim)?;
        sink.write_all(l)?;
        delim = DELIM_FOLLOW;
    }
    Ok(())
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
            if let Ok(re) = p.parse() {
                patterns.push(re);
            }
        }
        let mut pattern_strings = patterns.iter().map(|p| p.to_string()).collect::<Vec<_>>();
        let mut log_pattern = cli.log_pattern.unwrap_or_else(default_log_pattern);
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
            files.push(STDIN_FILENAME.to_owned())
        }
        // no-filename wins, otherwise if requested or multi-file
        let filenames = if cli.no_filename {
            false
        } else {
            cli.filename || files.len() > 1
        };
        Handler {
            files,
            pattern_set: RegexSet::new(&pattern_strings).unwrap(),
            max_count: cli.max_count,
            invert_match: cli.invert_match,
            counts: cli.count,
            color_mode: cli.color,
            stdin_label: cli.label,
            log_pattern,
            start,
            end,
            filenames,
        }
    }
}

impl Handler {
    #[cfg(test)]
    fn empty() -> Handler {
        Handler {
            files: Vec::new(),
            pattern_set: RegexSet::new([r"a"]).unwrap(),
            max_count: None,
            invert_match: false,
            counts: false,
            color_mode: ColorChoice::Auto,
            stdin_label: None,
            log_pattern: DEFAULT_LOG_PATTERN.parse().unwrap(),
            start: None,
            end: None,
            filenames: false,
        }
    }
}

#[cfg(test)]
mod from_cli_tests;

#[cfg(test)]
mod handler_tests;
