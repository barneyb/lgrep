use std::fmt::{Display, Formatter};
use std::io::Cursor;

use super::*;

const APP_LOG: &str = include_str!("../../app.log");
const RECORD_DRAINING: &str = include_str!("../../record_draining.log");
const RECORD_WITH_TRACE: &str = include_str!("../../record_with_trace.log");
const RECORD_COMPLETE: &str = include_str!("../../record_complete.log");
const RECORD_UNRELATED: &str = include_str!("../../record_unrelated.log");

impl Handler {
    fn all_re() -> Handler {
        Handler {
            pattern_set: RegexSet::new([r"P", r"Q", r"R"]).unwrap(),
            log_pattern: r"L".parse().unwrap(),
            start: Some(r"S".parse().unwrap()),
            end: Some(r"E".parse().unwrap()),
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
fn is_match() {
    let h = Handler::all_re();
    assert!(h.is_match("0P0"));
    assert!(!h.is_match("zzz"));
    assert!(h.is_match("0Q0"));
    assert!(!h.is_match("zzz"));
    assert!(h.is_match("0R0"));
    assert!(!h.is_match("zzz"));
}

#[test]
fn is_record_start() {
    let h = Handler::all_re();
    assert!(h.is_record_start("0L0"));
    assert!(!h.is_record_start("zzz"));
}

#[test]
fn is_record_start_default() {
    let h = Handler::empty();
    assert!(
        h.is_record_start("2024-07-01 01:25:47.755 Unexpected error occurred in scheduled task")
    );
    assert!(!h.is_record_start("    at org.springframework.orm.jpa.JpaTransactionManager.doBegin(JpaTransactionManager.java:466)"));
}

#[test]
fn is_record_start_custom() {
    let h = Handler {
        log_pattern: "GOAT".parse().unwrap(),
        ..Handler::empty()
    };
    assert!(h.is_record_start("i am a GOAT or something?"));
    assert!(!h.is_record_start("definitely only a rabbit"));
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

// 'static here is a kludge, but it's just for tests, so meh
impl MatchesAndCount {
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
        let mut sink = BufWriter::new(mac);
        let exit = Some(handler.process_file(source, &mut sink).unwrap());
        mac = sink.into_inner().unwrap();
        mac.exit = exit;
        mac
    }
}

#[test]
fn app_log_for_error() {
    let handler = Handler {
        pattern_set: RegexSet::new([r"(?i)error"]).unwrap(),
        ..Handler::empty()
    };
    let mac = MatchesAndCount::run(&handler, APP_LOG);
    assert_eq!(vec![RECORD_WITH_TRACE, RECORD_COMPLETE,], mac.records);
}

#[test]
fn app_log_for_transaction() {
    let handler = Handler {
        pattern_set: RegexSet::new([r"startTransaction"]).unwrap(),
        ..Handler::empty()
    };
    let mac = MatchesAndCount::run(&handler, APP_LOG);
    assert_eq!(vec![RECORD_WITH_TRACE], mac.records);
}

#[test]
fn simple_process_file() {
    let handler = Handler {
        pattern_set: RegexSet::new([r"t"]).unwrap(),
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
        pattern_set: RegexSet::new([r"(?i)error"]).unwrap(),
        start: Some(r"QueueProcessor".parse().unwrap()), // middle of the trace
        ..Handler::empty()
    };
    let mac = MatchesAndCount::run(&handler, APP_LOG);
    assert_eq!(vec![RECORD_WITH_TRACE, RECORD_COMPLETE], mac.records);
}

#[test]
fn app_log_end() {
    let handler = Handler {
        pattern_set: RegexSet::new([r"(?i)queue"]).unwrap(),
        end: Some(r"QueueProcessor".parse().unwrap()),
        ..Handler::empty()
    };
    let mac = MatchesAndCount::run(&handler, APP_LOG);
    assert_eq!(vec![RECORD_DRAINING], mac.records);
}

#[test]
fn app_log_final_line() {
    let handler = Handler {
        pattern_set: RegexSet::new([r"unrelated"]).unwrap(),
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
        pattern_set: RegexSet::new([r"o"]).unwrap(),
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
        "spiffy.txt:one\nspiffy.txt:two\nspiffy.txt:four",
        mac.to_string()
    );
    assert_eq!(3, mac.flush_count);
    assert_eq!(Some(Exit::Match), mac.exit);
}

#[test]
fn filenames_multiline_records() {
    let handler = Handler {
        pattern_set: RegexSet::new([r"r"]).unwrap(),
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
four",
    );
    assert_eq!("spiffy.txt:three\nspiffy.txt-four", mac.to_string());
    assert_eq!(1, mac.flush_count);
    assert_eq!(Some(Exit::Match), mac.exit);
}

#[test]
fn filenames_final_newline() {
    let handler = Handler {
        pattern_set: RegexSet::new([r"r"]).unwrap(),
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
    assert_eq!("spiffy.txt:three\nspiffy.txt-four\n", mac.to_string());
    assert_eq!(1, mac.flush_count);
    assert_eq!(Some(Exit::Match), mac.exit);
}

#[test]
fn max_count() {
    let handler = Handler {
        pattern_set: RegexSet::new([r"t", r"u"]).unwrap(),
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
        pattern_set: RegexSet::new([r"ee"]).unwrap(),
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
            "LOG: three\nfour\n"
        ],
        mac.records
    );
    assert_eq!(3, mac.flush_count);
    assert_eq!(Some(Exit::Match), mac.exit);
}

#[test]
fn no_matches() {
    let handler = Handler {
        pattern_set: RegexSet::new([r"ZZZZZ"]).unwrap(),
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
        pattern_set: RegexSet::new([r"ZZZZ"]).unwrap(),
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
        pattern_set: RegexSet::new([r"ZZZZ"]).unwrap(),
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
        pattern_set: RegexSet::new([r"r"]).unwrap(),
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
        pattern_set: RegexSet::new([r"e"]).unwrap(),
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
        pattern_set: RegexSet::new([r"e"]).unwrap(),
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
