use std::env;
use std::io::{BufRead, BufWriter, ErrorKind, Write};

use anyhow::{Context, Error, Result};
use regex::{Regex, RegexSet};

use io::STDIN_FILENAME;

use crate::cli::Cli;
use crate::{io, Exit};

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
    stdin_label: String,
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

struct Source<'a> {
    filename: &'a str,
    reader: Box<dyn BufRead>,
}

type Sink = BufWriter<dyn Write>;

impl Handler {
    pub(crate) fn run(&self) -> Result<Exit> {
        let mut sink = BufWriter::new(std::io::stdout().lock());
        let mut exit = Exit::NoMatch;
        for f in self.files.iter() {
            #[rustfmt::skip]
            let mut source = Source {
                filename: if f == STDIN_FILENAME { &self.stdin_label } else { f },
                reader: io::get_reader(f)?,
            };
            match self.process_file(&mut source, &mut sink)? {
                Exit::Terminate => {
                    return Ok(Exit::Terminate);
                }
                Exit::Match => exit = Exit::Match,
                _ => {}
            }
        }
        Ok(exit)
    }

    fn process_file(&self, source: &mut Source, sink: &mut Sink) -> Result<Exit> {
        let mut file_started = !self.has_start();
        let mut before_first_record = true;
        let mut match_count = 0;
        // an entire log record
        let mut record = String::new();
        // a single line of input (w/ the newline, if present)
        let mut line = String::new();
        loop {
            // while let soaks up an Err; we want to propagate it
            let n = source.reader.read_line(&mut line)?;
            let is_eof = n == 0;
            let start_of_record = self.is_record_start(&line);
            if before_first_record && start_of_record {
                before_first_record = false;
            }
            if before_first_record || start_of_record || is_eof {
                // process the buffered record
                if self.is_end(&record) {
                    break; // reached end
                }
                if !file_started && self.is_start(&record) {
                    file_started = true;
                }
                if file_started && self.is_match(&record) {
                    if !self.counts {
                        self.write(sink, &record, source.filename)?;
                    }
                    match_count += 1;
                    if self.is_max_reached(match_count) {
                        break; // reached max count
                    }
                }
                if is_eof {
                    break; // reached EOF
                }
                // start a new record with line
                record.clone_from(&line);
            } else {
                // add line to the current record
                record.push_str(&line);
            }
            line.clear();
        }
        if self.counts {
            self.write(sink, &format!("{match_count}\n"), source.filename)?;
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
        let filename = if cli.no_filename {
            false
        } else {
            cli.filename || files.len() > 1
        };
        Handler {
            pattern_set: RegexSet::new(&pattern_strings).unwrap(),
            filenames: filename,
            files,
            max_count: cli.max_count,
            counts: cli.count,
            stdin_label: cli.label.unwrap_or_else(|| DEFAULT_STDIN_LABEL.to_owned()),
            invert_match: cli.invert_match,
            log_pattern,
            start,
            end,
        }
    }
}

#[cfg(test)]
mod handler_tests;
