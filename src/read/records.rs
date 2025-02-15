use regex_automata::meta::Regex;

use crate::read::lines::{Line, Lines};

pub(crate) struct Records<'a> {
    lines: Lines,
    log_pattern: &'a Regex,
    before_first_record: bool,
    record_num: usize,
    curr_line: Option<Line>,
}

impl<'a> Records<'a> {
    pub(crate) fn new(lines: Lines, log_pattern: &Regex) -> Records {
        Records {
            lines,
            log_pattern,
            before_first_record: true,
            record_num: 0,
            curr_line: None,
        }
    }

    fn advance(&mut self) -> Option<anyhow::Result<Line>> {
        if let Some(l) = self.curr_line.take() {
            Some(Ok(l))
        } else {
            self.lines.next()
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Record {
    pub text: String,
    pub record_num: usize,
    pub first_line: usize,
}

impl Record {
    pub(crate) fn push_line(&mut self, line: &Line) {
        self.text.push('\n');
        self.text.push_str(&line.text);
    }
}

impl<'a> Iterator for Records<'a> {
    type Item = anyhow::Result<Record>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut record = match self.advance() {
            None => {
                return None;
            }
            Some(Err(e)) => {
                return Some(Err(e));
            }
            Some(Ok(l)) => {
                self.record_num += 1;
                if self.log_pattern.is_match(&l.text) {
                    self.before_first_record = false;
                }
                Record {
                    record_num: self.record_num,
                    first_line: l.line_num,
                    text: l.text,
                }
            }
        };
        for line in self.lines.by_ref() {
            match line {
                Err(e) => {
                    return Some(Err(e));
                }
                Ok(l) => {
                    if self.log_pattern.is_match(&l.text) {
                        self.before_first_record = false;
                        let _ = self.curr_line.insert(l);
                        break;
                    } else if self.before_first_record {
                        let _ = self.curr_line.insert(l);
                        break;
                    } else {
                        // add line to the current record
                        record.push_line(&l);
                    }
                }
            }
        }
        Some(Ok(record))
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use super::*;

    impl Record {
        pub(crate) fn new(text: &str, record_num: usize, first_line: usize) -> Record {
            Record {
                text: text.to_owned(),
                record_num,
                first_line,
            }
        }
    }

    fn to_records(text: &'static str, re: &Regex) -> Vec<Record> {
        Lines::new(Box::new(Cursor::new(text)))
            .records(re)
            .map(|r| r.unwrap())
            .collect::<Vec<_>>()
    }

    #[test]
    fn does_it_smoke() {
        let re = Regex::new("o").unwrap();
        assert_eq!(
            vec![
                Record::new("one\nzzzz", 1, 1),
                Record::new("two\nthree", 2, 3),
                Record::new("four\nfive", 3, 5),
            ],
            to_records(
                "one\nzzzz
two\nthree
four\nfive",
                &re
            )
        )
    }

    #[test]
    fn before_first_log_record() {
        // before the first log record boundary, treat every line as its own record
        let re = Regex::new(r"LOG").unwrap();
        assert_eq!(
            vec![
                Record::new("one, thee father", 1, 1),
                Record::new("two, thee mother", 2, 2),
                Record::new("LOG: three\nfour", 3, 3),
                Record::new("LOG: five\nsix", 4, 5),
            ],
            to_records(
                "one, thee father
two, thee mother
LOG: three\nfour
LOG: five\nsix
",
                &re
            )
        )
    }
}
