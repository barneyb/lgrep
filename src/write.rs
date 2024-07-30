use std::io::{BufWriter, ErrorKind, Write};

use anyhow::{Context, Error, Result};
use regex_automata::meta::FindMatches;

use crate::read::records::Record;
use crate::write::capabilities::Capabilities;
use crate::Exit;

pub(crate) mod capabilities;

const FLUSH_BUFFER_AT: usize = 8192;

type Sink = BufWriter<dyn Write>;

macro_rules! styled {
    ($dst:expr, $opt_style:expr, $arg:expr) => {
        if let Some(s) = &$opt_style {
            write!($dst, "{}{}{0:#}", s, $arg)
        } else {
            write!($dst, "{}", $arg)
        }
    };
}

// todo: split this up based on the style of output
pub(crate) struct LgrepWrite<'a> {
    capabilities: Option<Capabilities>,
    filenames: bool,
    line_numbers: bool,
    sink: &'a mut Sink,
}

impl<'a> LgrepWrite<'a> {
    pub(crate) fn new(
        colorize: bool,
        filenames: bool,
        line_numbers: bool,
        sink: &'a mut Sink,
    ) -> LgrepWrite<'a> {
        LgrepWrite {
            capabilities: if colorize {
                Some(Capabilities::from_env())
            } else {
                None
            },
            filenames,
            line_numbers,
            sink,
        }
    }

    pub(crate) fn needs_match_locations(&self) -> bool {
        if let Some(cs) = &self.capabilities {
            cs.match_text.is_some()
        } else {
            false
        }
    }

    pub(crate) fn write_count(&mut self, filename: &str, count: usize) -> Result<Exit> {
        debug_assert!(
            !self.line_numbers,
            "line numbers and counts together makes no sense"
        );
        self.spew(filename, &format!("{count}\n"), 0)
    }

    pub(crate) fn write_record_with_matches(
        &mut self,
        filename: &str,
        record: &Record,
        matches: FindMatches,
    ) -> Result<Exit> {
        if let Some(cs) = &self.capabilities {
            if let Some(s) = cs.match_text {
                // allocate a little extra space, so a single match probably won't reallocate.
                let mut text = String::with_capacity(record.text.len() + 20);
                let mut thru = 0;
                for m in matches {
                    if m.start() > thru {
                        text.push_str(&record.text[thru..m.start()]);
                    }
                    text.push_str(&format!("{}{}{0:#}", s, &record.text[m.start()..m.end()]));
                    thru = m.end();
                }
                if thru < record.text.len() {
                    text.push_str(&record.text[thru..])
                }
                return self.spew(filename, &text, record.first_line);
            }
        }
        debug_assert!(false, "write_record_with_matches invoked w/ no styling?!");
        self.write_record(filename, record)
    }

    pub(crate) fn write_record(&mut self, filename: &str, record: &Record) -> Result<Exit> {
        self.spew(filename, &record.text, record.first_line)
    }

    fn spew(&mut self, filename: &str, text: &str, first_line: usize) -> Result<Exit> {
        let r = self
            .spew_internal(filename, text, first_line)
            .and_then(|_| self.sink.flush());
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

    fn spew_internal(
        &mut self,
        filename: &str,
        text: &str,
        first_line: usize,
    ) -> std::io::Result<()> {
        let lines = text.split_inclusive('\n');
        let mut separator = ':';
        let mut line_num = first_line;
        for l in lines {
            if let Some(cs) = &self.capabilities {
                if self.filenames {
                    styled!(self.sink, cs.filename, filename)?;
                    styled!(self.sink, cs.separator, separator)?;
                }
                if self.line_numbers {
                    styled!(self.sink, cs.line_number, line_num)?;
                    styled!(self.sink, cs.separator, separator)?;
                }
            } else {
                if self.filenames {
                    write!(self.sink, "{filename}")?;
                    write!(self.sink, "{separator}")?;
                }
                if self.line_numbers {
                    write!(self.sink, "{line_num}")?;
                    write!(self.sink, "{separator}")?;
                }
            }
            write!(self.sink, "{l}")?;
            if self.sink.buffer().len() >= FLUSH_BUFFER_AT {
                self.sink.flush()?
            }
            separator = '-';
            line_num += 1;
        }
        Ok(())
    }
}
