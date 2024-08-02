use std::env;
use std::io::BufWriter;

use anyhow::{Context, Result};
use clap::ColorChoice;
use is_terminal::is_terminal;
use regex_automata::meta::Regex;
use regex_automata::util::syntax;

use read::STDIN_FILENAME;

use crate::cli::Cli;
use crate::read::source::Source;
use crate::write::LgrepWrite;
use crate::{read, Exit};

const ENV_LOG_PATTERN: &str = "LGREP_LOG_PATTERN";

const DEFAULT_LOG_PATTERN: &str = r"(^|:)\d{4}-\d{2}-\d{2}[ T]\d{2}:\d{2}:\d{2}[.,]\d";
const DEFAULT_STDIN_LABEL: &str = "(standard input)";

pub(crate) struct Handler {
    files: Vec<String>,
    pattern_set: Regex,
    max_count: Option<usize>,
    invert_match: bool,
    counts: bool,
    color_mode: ColorChoice,
    quiet: bool,
    stdin_label: Option<String>,
    log_pattern: Regex,
    start: Option<Regex>,
    end: Option<Regex>,
    filenames: bool,
    line_numbers: bool,
}

fn opt_re_match(opt_re: &Option<Regex>, hay: &str) -> bool {
    if let Some(re) = &opt_re {
        re.is_match(hay)
    } else {
        false
    }
}

impl Handler {
    pub(crate) fn run(&self) -> Result<Exit> {
        let lock = std::io::stdout().lock();
        let colorize = match self.color_mode {
            ColorChoice::Auto => is_terminal(&lock),
            ColorChoice::Always => true,
            ColorChoice::Never => false,
        };
        let mut sink = BufWriter::new(lock);
        let mut write = LgrepWrite::new(colorize, self.filenames, self.line_numbers, &mut sink);
        self.run_with(&mut write)
    }

    fn run_with(&self, sink: &mut LgrepWrite) -> Result<Exit> {
        let mut exit = Exit::NoMatch;
        for f in self.files.iter() {
            let reader = read::get_reader(f)?;
            let source = Source::new(self.display_name_for_filename(f), reader);
            match self.process_file(source, sink)? {
                Exit::Terminate => {
                    exit = Exit::Terminate;
                    break;
                }
                Exit::Match => {
                    exit = Exit::Match;
                    if self.quiet {
                        break;
                    }
                }
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

    fn process_file(&self, source: Source, sink: &mut LgrepWrite) -> Result<Exit> {
        let mut file_started = !self.has_start();
        let mut match_count = 0;
        let filename = source.filename;
        let needs_matches = !self.invert_match && sink.needs_match_locations();
        // an entire log record
        for record in source.records(&self.log_pattern) {
            // while let soaks up an Err; we want to propagate it
            match record {
                Err(e) => {
                    return Err(e).with_context(|| format!("Failed to read from '{}'", filename))
                }
                Ok(r) => {
                    if self.is_end(&r.text) {
                        break;
                    }
                    if !file_started {
                        if self.is_start(&r.text) {
                            file_started = true;
                        } else {
                            continue;
                        }
                    }
                    if self.invert_match ^ self.pattern_set.is_match(&r.text) {
                        if !self.counts && !self.quiet {
                            if needs_matches {
                                sink.write_record_with_matches(
                                    filename,
                                    &r,
                                    self.pattern_set.find_iter(&r.text),
                                )?;
                            } else {
                                sink.write_record(filename, &r)?;
                            }
                        }
                        match_count += 1;
                        if self.is_max_reached(match_count) {
                            break; // reached max count
                        }
                    }
                }
            }
        }
        if self.counts {
            sink.write_count(filename, match_count)?;
        }
        Ok(Exit::from(match_count))
    }

    fn is_max_reached(&self, match_count: usize) -> bool {
        self.quiet
            || if let Some(mc) = self.max_count {
                match_count >= mc
            } else {
                false
            }
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
}

impl Handler {
    pub(crate) fn new(cli: Cli) -> Result<Handler> {
        let mut re_builder = Regex::builder();
        if cli.ignore_case {
            re_builder.syntax(syntax::Config::new().case_insensitive(true));
        }
        let mut patterns = cli.patterns;
        if let Some(p) = cli.pattern {
            patterns.push(p);
        }
        let log_pattern = if let Some(p) = cli.log_pattern {
            re_builder.build(&p)?
        } else if let Ok(p) = env::var(ENV_LOG_PATTERN) {
            re_builder.build(&p)?
        } else {
            re_builder.build(DEFAULT_LOG_PATTERN)?
        };
        let start = if let Some(p) = cli.start {
            Some(re_builder.build(&p)?)
        } else {
            None
        };
        let end = if let Some(p) = cli.end {
            Some(re_builder.build(&p)?)
        } else {
            None
        };
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
        Ok(Handler {
            files,
            pattern_set: re_builder.build_many(&patterns)?,
            max_count: cli.max_count,
            invert_match: cli.invert_match,
            counts: cli.count,
            line_numbers: cli.line_number && !cli.count,
            color_mode: cli.color,
            quiet: cli.quiet,
            stdin_label: cli.label,
            log_pattern,
            start,
            end,
            filenames,
        })
    }
}

impl Handler {
    #[cfg(test)]
    fn empty() -> Handler {
        Handler {
            files: Vec::new(),
            pattern_set: Regex::new_many(&[r"a"]).unwrap(),
            max_count: None,
            invert_match: false,
            counts: false,
            color_mode: ColorChoice::Auto,
            quiet: false,
            stdin_label: None,
            log_pattern: Regex::new(DEFAULT_LOG_PATTERN).unwrap(),
            start: None,
            end: None,
            filenames: false,
            line_numbers: false,
        }
    }
}

/// Assert a Regex is as it should be, based on the passed match and non-match
/// lists of haystacks.
#[cfg(test)]
fn assert_re(re: &Regex, matches: &[&str], non_matches: &[&str]) {
    for m in matches {
        assert!(re.is_match(m), "Should have matched '{m}', but didn't");
    }
    for m in non_matches {
        assert!(!re.is_match(m), "Shouldn't have matched '{m}', but did");
    }
}

#[cfg(test)]
mod from_cli_tests;

#[cfg(test)]
mod handler_tests;
