use std::io::BufRead;

use regex_automata::meta::Regex;

use crate::read::records::Records;

pub(crate) struct Lines {
    reader: Box<dyn BufRead>,
    line_num: usize,
    eof: bool,
}

impl Lines {
    pub(crate) fn new(reader: Box<dyn BufRead>) -> Lines {
        Lines {
            reader,
            line_num: 0,
            eof: false,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Line {
    pub text: String,
    pub line_num: usize,
}

impl Iterator for Lines {
    type Item = anyhow::Result<Line>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.eof {
            return None;
        }
        let mut text = String::new();
        match self.reader.read_line(&mut text) {
            Err(e) => Some(Err(e.into())),
            Ok(n) => {
                if n == 0 {
                    self.eof = true;
                    return None;
                }
                if text.ends_with('\n') {
                    text.pop();
                }
                self.line_num += 1;
                Some(Ok(Line {
                    text,
                    line_num: self.line_num,
                }))
            }
        }
    }
}

impl Lines {
    pub(crate) fn records(self, log_pattern: &Regex) -> Records<'_> {
        Records::new(self, log_pattern)
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use super::*;

    impl Line {
        pub(crate) fn new(text: &str, line_num: usize) -> Line {
            Line {
                text: text.to_owned(),
                line_num,
            }
        }
    }

    #[test]
    fn does_it_smoke() {
        let lines: Vec<_> = Lines::new(Box::new(Cursor::new("one\ntwo\nthree")))
            .map(|r| r.unwrap())
            .collect();
        assert_eq!(
            vec![
                Line::new("one", 1),
                Line::new("two", 2),
                Line::new("three", 3),
            ],
            lines
        )
    }
}
