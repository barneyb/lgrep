use std::io::BufRead;

use regex::Regex;

use crate::read::lines::Lines;
use crate::read::records::Records;

pub(crate) struct Source<'a> {
    pub filename: &'a str,
    reader: Box<dyn BufRead>,
}

impl<'a> Source<'a> {
    pub(crate) fn new(filename: &str, reader: Box<dyn BufRead>) -> Source {
        Source { filename, reader }
    }

    pub(crate) fn lines(self) -> Lines {
        Lines::new(self.reader)
    }

    pub(crate) fn records(self, log_pattern: &Regex) -> Records {
        Records::new(self.lines(), log_pattern)
    }
}
