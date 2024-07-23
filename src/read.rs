use std::io::BufRead;

use anyhow::{Context, Result};

pub(crate) const STDIN_FILENAME: &str = "-";

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
}

pub(crate) struct Lines {
    reader: Box<dyn BufRead>,
    line_num: usize,
    eof: bool,
}

impl Lines {
    fn new(reader: Box<dyn BufRead>) -> Lines {
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
    type Item = Result<Line>;

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
                self.line_num += 1;
                Some(Ok(Line {
                    text,
                    line_num: self.line_num,
                }))
            }
        }
    }
}

/// Open a [BufRead] for the named file, or STDIN if the filename is '-'. If the
/// stream is compressed using a well-known format (e.g. gzip), it will be
/// decompressed automatically _on Unix-ish platforms_, by shelling out to an
/// appropriate utility on your `$PATH`. On Windows, you must manually
/// decompress the stream/file first.
pub(crate) fn get_reader(filename: &String) -> Result<Box<dyn BufRead>> {
    if filename == STDIN_FILENAME {
        open_stdin().with_context(|| "Failed to open STDIN for reading")
    } else {
        open_file(filename).with_context(|| format!("Failed to open '{filename}' for reading"))
    }
}

#[cfg(not(target_os = "windows"))]
fn open_stdin() -> Result<Box<dyn BufRead>> {
    use compress_io::compress::CompressIo;
    Ok(Box::new(CompressIo::new().bufreader()?))
}

#[cfg(not(target_os = "windows"))]
fn open_file(filename: &String) -> Result<Box<dyn BufRead>> {
    use compress_io::compress::CompressIo;
    Ok(Box::new(CompressIo::new().path(filename).bufreader()?))
}

#[cfg(target_os = "windows")]
fn open_stdin() -> Result<Box<dyn BufRead>> {
    use std::io::stdin;
    Ok(Box::new(stdin().lock()))
}

#[cfg(target_os = "windows")]
fn open_file(filename: &String) -> Result<Box<dyn BufRead>> {
    use std::fs::File;
    use std::io::BufReader;
    Ok(Box::new(BufReader::new(File::open(filename)?)))
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use super::*;

    impl Line {
        fn new(text: &str, line_num: usize) -> Line {
            Line {
                text: text.to_owned(),
                line_num,
            }
        }
    }

    #[test]
    fn does_lines_smoke() {
        let lines: Vec<_> = Lines::new(Box::new(Cursor::new("one\ntwo\nthree")))
            .map(|r| r.unwrap())
            .collect();
        assert_eq!(
            vec![
                Line::new("one\n", 1),
                Line::new("two\n", 2),
                Line::new("three", 3)
            ],
            lines
        )
    }
}
