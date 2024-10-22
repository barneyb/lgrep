use std::fmt::{Display, Formatter};
use std::io::Cursor;
use std::io::{BufWriter, Write};

use clap::ColorChoice;

use super::*;

const APP_LOG: &str = include_str!("../../app.log");
const RECORD_DRAINING: &str = include_str!("../../record_draining.log");
const RECORD_WITH_TRACE: &str = include_str!("../../record_with_trace.log");
const RECORD_COMPLETE: &str = include_str!("../../record_complete.log");
const RECORD_UNRELATED: &str = include_str!("../../record_unrelated.log");

impl Handler {
    fn all_re() -> Handler {
        let patterns = [r"P", r"Q", r"R"];
        Handler {
            pattern_set: Regex::new_many(&patterns).unwrap(),
            log_pattern: Regex::new(r"L").unwrap(),
            start: Some(Regex::new(r"S").unwrap()),
            end: Some(Regex::new(r"E").unwrap()),
            ..Self::empty()
        }
    }
}

#[test]
fn no_start() {
    let h = Handler::empty();
    assert!(!h.has_start());
}

#[test]
fn with_start() {
    let h = Handler::all_re();
    assert!(h.has_start());
}

#[test]
fn no_end() {
    let h = Handler::empty();
    assert!(!h.has_end());
}

#[test]
fn with_end() {
    let h = Handler::all_re();
    assert!(h.has_end());
}

#[test]
fn is_record_start() {
    let h = Handler::all_re();
    assert_re(&h.log_pattern, &["0L0"], &["zzz"]);
}

#[test]
fn is_record_start_default() {
    let h = Handler::empty();
    assert_re(&h.log_pattern,
              &["2024-07-01 01:25:47.755 Unexpected error occurred in scheduled task"],
              &["    at org.springframework.orm.jpa.JpaTransactionManager.doBegin(JpaTransactionManager.java:466)"]);
}

#[test]
fn is_record_start_custom() {
    let h = Handler {
        log_pattern: Regex::new("GOAT").unwrap(),
        ..Handler::empty()
    };
    assert_re(
        &h.log_pattern,
        &["i am a GOAT or something?"],
        &["definitely only a rabbit"],
    );
}

#[test]
fn is_start_none() {
    let h = Handler::empty();
    assert!(!h.is_start("0S0"));
}

#[test]
fn is_start() {
    let h = Handler::all_re();
    assert!(h.is_start("0S0"));
    assert!(!h.is_start("zzz"));
}

#[test]
fn is_end_none() {
    let h = Handler::empty();
    assert!(!h.is_end("0E0"));
}

#[test]
fn is_end() {
    let h = Handler::all_re();
    assert!(h.is_end("0E0"));
    assert!(!h.is_end("zzz"));
}

#[derive(Default, Debug)]
struct MatchesAndCount {
    records: Vec<String>,
    flush_count: usize,
    exit: Option<Exit>,
}

impl Display for MatchesAndCount {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for r in self.records.iter() {
            f.write_str(r)?;
        }
        Ok(())
    }
}

impl Write for MatchesAndCount {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.records.push(String::from_utf8_lossy(buf).to_string());
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.flush_count += 1;
        Ok(())
    }
}

impl MatchesAndCount {
    // 'static here is a kludge, but it's just for tests, so meh
    fn run(handler: &Handler, source: &'static str) -> MatchesAndCount {
        Self::run_with_filename(handler, "input.txt", source)
    }

    fn run_with_filename(
        handler: &Handler,
        filename: &str,
        source: &'static str,
    ) -> MatchesAndCount {
        let source = Source::new(filename, Box::new(Cursor::new(source.as_bytes())));
        let mut mac = MatchesAndCount::default();
        let mut buf_writer = BufWriter::new(mac);
        let mut write = LgrepWrite::new(
            handler.color_mode == ColorChoice::Always,
            handler.filenames,
            handler.line_numbers,
            &mut buf_writer,
        );
        let exit = Some(handler.process_file(source, &mut write).unwrap());
        mac = buf_writer.into_inner().unwrap();
        mac.exit = exit;
        mac
    }
}

#[test]
fn app_log_for_error() {
    let handler = Handler {
        pattern_set: Regex::new(r"(?i)error").unwrap(),
        ..Handler::empty()
    };
    let mac = MatchesAndCount::run(&handler, APP_LOG);
    assert_eq!(vec![RECORD_WITH_TRACE, RECORD_COMPLETE,], mac.records);
}

#[test]
fn app_log_for_not_error() {
    let handler = Handler {
        pattern_set: Regex::new(r"(?i)error").unwrap(),
        invert_match: true,
        ..Handler::empty()
    };
    let mac = MatchesAndCount::run(&handler, APP_LOG);
    assert_eq!(vec![RECORD_DRAINING, RECORD_UNRELATED,], mac.records);
}

#[test]
fn app_log_for_transaction() {
    let handler = Handler {
        pattern_set: Regex::new(r"startTransaction").unwrap(),
        ..Handler::empty()
    };
    let mac = MatchesAndCount::run(&handler, APP_LOG);
    assert_eq!(vec![RECORD_WITH_TRACE], mac.records);
}

#[test]
fn simple_process_file() {
    let handler = Handler {
        pattern_set: Regex::new(r"t").unwrap(),
        log_pattern: Regex::new(r".").unwrap(),
        ..Handler::empty()
    };
    let mac = MatchesAndCount::run(
        &handler,
        "line one
line two
third line
line 4
",
    );
    assert_eq!(vec!["line two\n", "third line\n"], mac.records);
}

#[test]
fn app_log_start() {
    let handler = Handler {
        pattern_set: Regex::new(r"(?i)error").unwrap(),
        start: Some(Regex::new(r"QueueProcessor").unwrap()), // middle of the trace
        ..Handler::empty()
    };
    let mac = MatchesAndCount::run(&handler, APP_LOG);
    assert_eq!(vec![RECORD_WITH_TRACE, RECORD_COMPLETE], mac.records);
}

#[test]
fn app_log_end() {
    let handler = Handler {
        pattern_set: Regex::new(r"(?i)queue").unwrap(),
        end: Some(Regex::new("QueueProcessor").unwrap()),
        ..Handler::empty()
    };
    let mac = MatchesAndCount::run(&handler, APP_LOG);
    assert_eq!(vec![RECORD_DRAINING], mac.records);
}

#[test]
fn app_log_final_line() {
    let handler = Handler {
        pattern_set: Regex::new(r"unrelated").unwrap(),
        ..Handler::empty()
    };
    let mac = MatchesAndCount::run(&handler, APP_LOG);
    assert_eq!(vec![RECORD_UNRELATED], mac.records);
}

#[test]
fn display_name_for_named_file() {
    let handler = Handler { ..Handler::empty() };
    assert_eq!(
        "spiffy.log",
        handler.display_name_for_filename("spiffy.log")
    )
}

#[test]
fn display_name_for_stdin() {
    let handler = Handler { ..Handler::empty() };
    assert_eq!("(standard input)", handler.display_name_for_filename("-"))
}

#[test]
fn display_name_for_labeled_stdin() {
    let handler = Handler {
        stdin_label: Some("Johann".to_string()),
        ..Handler::empty()
    };
    assert_eq!("Johann", handler.display_name_for_filename("-"))
}

#[test]
fn filenames_singleline_records() {
    let handler = Handler {
        pattern_set: Regex::new(r"o").unwrap(),
        log_pattern: Regex::new(r".").unwrap(),
        filenames: true,
        ..Handler::empty()
    };
    let mac = MatchesAndCount::run_with_filename(
        &handler,
        "spiffy.txt",
        "one
two
three
four",
    );
    assert_eq!(
        vec!["spiffy.txt:one\n", "spiffy.txt:two\n", "spiffy.txt:four\n",],
        mac.records
    );
    assert_eq!(3, mac.flush_count);
    assert_eq!(Some(Exit::Match), mac.exit);
}

#[test]
fn filenames_multiline_records() {
    let handler = Handler {
        pattern_set: Regex::new(r"r").unwrap(),
        log_pattern: Regex::new(r"e").unwrap(),
        filenames: true,
        line_numbers: true,
        ..Handler::empty()
    };
    let mac = MatchesAndCount::run_with_filename(
        &handler,
        "spiffy.txt",
        "one
two
three
four",
    );
    assert_eq!(vec!["spiffy.txt:3:three\nspiffy.txt-4-four\n"], mac.records);
    assert_eq!(1, mac.flush_count);
    assert_eq!(Some(Exit::Match), mac.exit);
}

#[test]
fn colors() {
    let handler = Handler {
        pattern_set: Regex::new(r"r").unwrap(),
        log_pattern: Regex::new(r"e").unwrap(),
        filenames: true,
        line_numbers: true,
        color_mode: ColorChoice::Always,
        ..Handler::empty()
    };
    let mac = MatchesAndCount::run_with_filename(
        &handler,
        "spiffy.txt",
        "one
two
three
four'n'stuff",
    );
    assert_eq!(
        vec![
        "\u{1b}[35mspiffy.txt\u{1b}[0m\u{1b}[36m:\u{1b}[0m\u{1b}[32m3\u{1b}[0m\u{1b}[36m:\u{1b}[0mth\u{1b}[1m\u{1b}[31mr\u{1b}[0mee
\u{1b}[35mspiffy.txt\u{1b}[0m\u{1b}[36m-\u{1b}[0m\u{1b}[32m4\u{1b}[0m\u{1b}[36m-\u{1b}[0mfou\u{1b}[1m\u{1b}[31mr\u{1b}[0m'n'stuff
"],
        mac.records
    );
    assert_eq!(1, mac.flush_count);
    assert_eq!(Some(Exit::Match), mac.exit);
}

#[test]
fn color_multiline_match() {
    let handler = Handler {
        pattern_set: Regex::new(r"XXX\nYYY").unwrap(),
        log_pattern: Regex::new(r"e").unwrap(),
        color_mode: ColorChoice::Always,
        ..Handler::empty()
    };
    let mac = MatchesAndCount::run(
        &handler,
        "one
two
threeXXX
YYYfour",
    );
    assert_eq!(
        vec![
            "three\u{1b}[1m\u{1b}[31mXXX\u{1b}[0m
\u{1b}[1m\u{1b}[31mYYY\u{1b}[0mfour
"
        ],
        mac.records
    );
    assert_eq!(1, mac.flush_count);
    assert_eq!(Some(Exit::Match), mac.exit);
}

#[test]
fn filenames_final_newline() {
    let handler = Handler {
        pattern_set: Regex::new(r"r").unwrap(),
        log_pattern: Regex::new(r"e").unwrap(),
        filenames: true,
        ..Handler::empty()
    };
    let mac = MatchesAndCount::run_with_filename(
        &handler,
        "spiffy.txt",
        "one
two
three
four
",
    );
    assert_eq!(vec!["spiffy.txt:three\nspiffy.txt-four\n"], mac.records);
    assert_eq!(1, mac.flush_count);
    assert_eq!(Some(Exit::Match), mac.exit);
}

#[test]
fn max_count() {
    let handler = Handler {
        pattern_set: Regex::new_many(&[r"t", r"u"]).unwrap(),
        log_pattern: Regex::new(r"").unwrap(),
        max_count: Some(2),
        ..Handler::empty()
    };
    let mac = MatchesAndCount::run(
        &handler,
        "one
two
three
four
",
    );
    assert_eq!(vec!["two\n", "three\n"], mac.records);
}

#[test]
fn before_first_log_record() {
    let handler = Handler {
        pattern_set: Regex::new(r"ee").unwrap(),
        log_pattern: Regex::new(r"LOG").unwrap(),
        ..Handler::empty()
    };
    // before the first log record boundary, treat every line as its own record
    let mac = MatchesAndCount::run(
        &handler,
        "one, thee father
two, thee mother
egads, bad dad!
LOG: three
four
LOG: five
six
",
    );
    assert_eq!(
        vec![
            "one, thee father\n",
            "two, thee mother\n",
            "LOG: three\nfour\n",
        ],
        mac.records
    );
    assert_eq!(3, mac.flush_count);
    assert_eq!(Some(Exit::Match), mac.exit);
}

#[test]
fn no_matches() {
    let handler = Handler {
        pattern_set: Regex::new(r"ZZZZZ").unwrap(),
        max_count: Some(2),
        ..Handler::empty()
    };
    let mac = MatchesAndCount::run(&handler, "input");
    assert!(mac.records.is_empty());
    assert_eq!(0, mac.flush_count);
    assert_eq!(Some(Exit::NoMatch), mac.exit);
}

#[test]
fn counts_zero() {
    let handler = Handler {
        counts: true,
        pattern_set: Regex::new(r"ZZZZ").unwrap(),
        ..Handler::empty()
    };
    let mac = MatchesAndCount::run(
        &handler,
        "one
two
three
four
",
    );
    assert_eq!("0\n", mac.to_string());
    assert_eq!(Some(Exit::NoMatch), mac.exit);
}

#[test]
fn counts_zero_file() {
    let handler = Handler {
        counts: true,
        pattern_set: Regex::new(r"ZZZZ").unwrap(),
        filenames: true,
        ..Handler::empty()
    };
    let mac = MatchesAndCount::run_with_filename(
        &handler,
        "sally.txt",
        "one
two
three
four
",
    );
    assert_eq!("sally.txt:0\n", mac.to_string());
    assert_eq!(Some(Exit::NoMatch), mac.exit);
}

#[test]
fn counts_some() {
    let handler = Handler {
        counts: true,
        pattern_set: Regex::new(r"r").unwrap(),
        ..Handler::empty()
    };
    let mac = MatchesAndCount::run(
        &handler,
        "one
two
three
four
",
    );
    assert_eq!("2\n", mac.to_string());
    assert_eq!(Some(Exit::Match), mac.exit);
}

#[test]
fn counts_some_max() {
    let handler = Handler {
        counts: true,
        pattern_set: Regex::new(r"e").unwrap(),
        max_count: Some(1),
        ..Handler::empty()
    };
    let mac = MatchesAndCount::run(
        &handler,
        "one
two
three
four
",
    );
    assert_eq!("1\n", mac.to_string());
    assert_eq!(Some(Exit::Match), mac.exit);
}

#[test]
fn counts_some_unreached_max() {
    let handler = Handler {
        counts: true,
        pattern_set: Regex::new(r"e").unwrap(),
        max_count: Some(99999),
        ..Handler::empty()
    };
    let mac = MatchesAndCount::run(
        &handler,
        "one
two
three
four
",
    );
    assert_eq!("2\n", mac.to_string());
    assert_eq!(Some(Exit::Match), mac.exit);
}

#[test]
fn quiet_match() {
    let handler = Handler {
        quiet: true,
        ..Handler::empty()
    };
    let mac = MatchesAndCount::run(&handler, "aleph\nbob\ncow\ndavid");
    assert_eq!("", mac.to_string());
    assert_eq!(Some(Exit::Match), mac.exit);
}

#[test]
fn quiet_no_match() {
    let handler = Handler {
        quiet: true,
        ..Handler::empty()
    };
    let mac = MatchesAndCount::run(&handler, "bob\ncow");
    assert_eq!("", mac.to_string());
    assert_eq!(Some(Exit::NoMatch), mac.exit);
}
