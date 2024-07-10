use std::env;
use std::io::{BufRead, ErrorKind, Write};

use anyhow::{Context, Result};
use regex::{Regex, RegexSet};

use crate::cli::Cli;
use crate::io;
use crate::io::STD_IN_FILENAME;
use crate::Exit::*;

const ENV_LOG_PATTERN: &str = "LGREP_LOG_PATTERN";

const DEFAULT_LOG_PATTERN: &str = r"(^|:)\d{4}-\d{2}-\d{2}[ T]\d{2}:\d{2}:\d{2}[.,]\d";
const DEFAULT_LABEL: &str = "(standard input)";

const INSENSITIVE_PREFIX: &str = "(?i)";

pub(crate) struct Handler {
    files: Vec<String>,
    pattern: RegexSet,
    max_count: Option<usize>,
    invert_match: bool,
    label: String,
    log_pattern: Regex,
    start: Option<Regex>,
    end: Option<Regex>,
    filename: bool,
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

impl Handler {
    pub(crate) fn run(&self) -> Result<crate::Exit> {
        let mut sink = std::io::stdout().lock();
        let mut exit = NoMatch;
        for f in self.files.iter() {
            let mut source = Source {
                filename: if f == STD_IN_FILENAME { &self.label } else { f },
                reader: io::get_reader(f)?,
            };
            match self.process_file(&mut source, &mut sink)? {
                Terminate => {
                    exit = Terminate;
                    break;
                }
                Match => exit = Match,
                _ => {}
            }
        }
        Ok(exit)
    }

    fn process_file(&self, source: &mut Source, sink: &mut dyn Write) -> Result<crate::Exit> {
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
                    let r = if self.filename {
                        with_filename(sink, &record, source.filename)
                    } else {
                        sink.write_all(record.as_bytes())
                    };
                    if let Err(e) = r {
                        return if e.kind() == ErrorKind::BrokenPipe {
                            // nothing is listening anymore
                            Ok(Terminate)
                        } else {
                            let msg = format!("Failed to write record: {e}");
                            Err(e).with_context(|| msg)
                        };
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
        Ok(if match_count == 0 { NoMatch } else { Match })
    }

    fn is_max_reached(&self, match_count: usize) -> bool {
        if let Some(mc) = self.max_count {
            match_count >= mc
        } else {
            false
        }
    }

    fn is_match(&self, hay: &str) -> bool {
        self.invert_match ^ self.pattern.is_match(hay)
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
}

fn default_log_pattern() -> Regex {
    if let Ok(p) = env::var(ENV_LOG_PATTERN) {
        p.parse()
    } else {
        DEFAULT_LOG_PATTERN.parse()
    }
    .unwrap()
}

fn with_filename(sink: &mut dyn Write, record: &String, filename: &str) -> std::io::Result<()> {
    let fn_bytes = filename.as_bytes();
    let lines = record.as_bytes().split_inclusive(|b| *b == b'\n');
    let mut started = false;
    for l in lines {
        sink.write_all(fn_bytes)?;
        sink.write_all(&[if started { b'-' } else { b':' }])?;
        sink.write_all(l)?;
        started = true;
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
            patterns.push(p);
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
mod handler_tests;
