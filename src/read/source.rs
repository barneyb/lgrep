use std::io::BufRead;

use regex_automata::meta::Regex;

use crate::read::lines::Lines;
use crate::read::records::Records;

pub(crate) struct Source<'a> {
    pub filename: &'a str,
    reader: Box<dyn BufRead>,
}

impl<'a> Source<'a> {
    pub(crate) fn new(filename: &str, reader: Box<dyn BufRead>) -> Source<'_> {
        Source { filename, reader }
    }

    pub(crate) fn lines(self) -> Lines {
        Lines::new(self.reader)
    }

    pub(crate) fn records(self, log_pattern: &Regex) -> Records<'_> {
        self.lines().records(log_pattern)
    }
}
