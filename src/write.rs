use std::fmt::Display;
use std::io::{BufWriter, ErrorKind, Write};

use anyhow::{Context, Error, Result};

use crate::read::records::Record;
use crate::write::capabilities::Capabilities;
use crate::Exit;

pub(crate) mod capabilities;

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

pub(crate) struct LgrepWrite<'a> {
    capabilities: Capabilities,
    filenames: bool,
    line_numbers: bool,
    sink: &'a mut Sink,
}

impl<'a> LgrepWrite<'a> {
    pub fn sink(
        colorize: bool,
        filenames: bool,
        line_numbers: bool,
        sink: &'a mut Sink,
    ) -> LgrepWrite<'a> {
        LgrepWrite {
            capabilities: if colorize {
                Capabilities::default() // todo: read from environment!
            } else {
                Capabilities::none()
            },
            filenames,
            line_numbers,
            sink,
        }
    }

    pub(crate) fn write_record(&mut self, filename: &str, record: &Record) -> Result<Exit> {
        let r = self
            .write_record_internal(filename, record)
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

    fn write_record_internal(&mut self, filename: &str, record: &Record) -> std::io::Result<()> {
        let lines = record.text.split_inclusive('\n');
        let mut separator = ':';
        let mut line_num = record.first_line;
        for l in lines {
            if self.filenames {
                styled!(self.sink, &self.capabilities.filename, filename)?;
                styled!(self.sink, &self.capabilities.separator, separator)?;
            }
            if self.line_numbers {
                styled!(self.sink, &self.capabilities.line_number, line_num)?;
                styled!(self.sink, &self.capabilities.separator, separator)?;
            }
            write!(self.sink, "{}{l}{0:#}", "")?;
            separator = '-';
            line_num += 1;
        }
        debug_assert_eq!(record.last_line, line_num - 1);
        Ok(())
    }
}
