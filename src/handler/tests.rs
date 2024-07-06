use std::fmt::{Display, Formatter};
use std::io::{Cursor, Error, ErrorKind};

use super::*;

const APP_LOG: &str = include_str!("../../app.log");
const LOG_WITH_TRACE: &str = "2024-07-01 01:25:47.755 Unexpected error occurred in scheduled task
org.springframework.transaction.CannotCreateTransactionException: Could not open JPA EntityManager for transaction
    at org.springframework.orm.jpa.JpaTransactionManager.doBegin(JpaTransactionManager.java:466)
    at org.springframework.transaction.support.AbstractPlatformTransactionManager.startTransaction(AbstractPlatformTransactionManager.java:531)
    at org.springframework.transaction.support.AbstractPlatformTransactionManager.getTransaction(AbstractPlatformTransactionManager.java:405)
    at org.springframework.transaction.support.TransactionTemplate.execute(TransactionTemplate.java:137)
    at com.brennaswitzer.cookbook.async.QueueProcessor.drainQueueInternal(QueueProcessor.java:68)
    ... many more frames ...
";

impl Handler {
    fn empty() -> Handler {
        Handler {
            files: Vec::new(),
            pattern: RegexSet::new(&[r"a"]).unwrap(),
            max_count: None,
            invert_match: false,
            label: DEFAULT_LABEL.to_owned(),
            log_pattern: DEFAULT_LOG_PATTERN.parse().unwrap(),
            start: None,
            end: None,
            filename: false,
        }
    }

    fn all_re() -> Handler {
        Handler {
            pattern: RegexSet::new(&[r"P", r"Q", r"R"]).unwrap(),
            log_pattern: r"T".parse().unwrap(),
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
    assert!(h.is_record_start("0T0"));
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
    count: usize,
}

impl Display for MatchesAndCount {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for r in self.records.iter() {
            f.write_str(&r)?;
        }
        Ok(())
    }
}

impl Write for MatchesAndCount {
    fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
        Err(Error::from(ErrorKind::Unsupported))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Err(Error::from(ErrorKind::Unsupported))
    }

    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.records.push(String::from_utf8_lossy(buf).to_string());
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
        let mut source = Source {
            filename,
            reader: Box::new(Cursor::new(source.as_bytes())),
        };
        let mut mac = MatchesAndCount::default();
        let count = handler.process_file(&mut source, &mut mac).unwrap();
        if !handler.filename {
            // w/ filenames, we'll get three writes per line, not one per record
            assert_eq!(count, mac.records.len());
        }
        mac.count = count;
        mac
    }
}

#[test]
fn app_log_for_error() {
    let handler = Handler {
        pattern: RegexSet::new(&[r"(?i)error"]).unwrap(),
        ..Handler::empty()
    };
    let mac = MatchesAndCount::run(&handler, APP_LOG);
    assert_eq!(
        vec![
            LOG_WITH_TRACE,
            "2024-07-01 01:25:47.790 queue draining complete (ERROR)\n"
        ],
        mac.records
    );
}

#[test]
fn app_log_for_transaction() {
    let handler = Handler {
        pattern: RegexSet::new(&[r"startTransaction"]).unwrap(),
        ..Handler::empty()
    };
    let mac = MatchesAndCount::run(&handler, APP_LOG);
    assert_eq!(vec![LOG_WITH_TRACE], mac.records);
}

#[test]
fn simple_process_file() {
    let handler = Handler {
        pattern: RegexSet::new(&[r"t"]).unwrap(),
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
        pattern: RegexSet::new(&[r"(?i)error"]).unwrap(),
        start: Some(r"QueueProcessor".parse().unwrap()), // middle of the trace
        ..Handler::empty()
    };
    let mac = MatchesAndCount::run(&handler, APP_LOG);
    assert_eq!(
        vec![
            LOG_WITH_TRACE,
            "2024-07-01 01:25:47.790 queue draining complete (ERROR)\n"
        ],
        mac.records
    );
}

#[test]
fn app_log_end() {
    let handler = Handler {
        pattern: RegexSet::new(&[r"(?i)queue"]).unwrap(),
        end: Some(r"QueueProcessor".parse().unwrap()),
        ..Handler::empty()
    };
    let mac = MatchesAndCount::run(&handler, APP_LOG);
    assert_eq!(
        vec!["2024-07-01 01:25:46.123 draining queue\n"],
        mac.records
    );
}

#[test]
fn app_log_final_line() {
    let handler = Handler {
        pattern: RegexSet::new(&[r"unrelated"]).unwrap(),
        ..Handler::empty()
    };
    let mac = MatchesAndCount::run(&handler, APP_LOG);
    assert_eq!(
        vec!["2024-07-01 01:25:48.000 some other unrelated log message\n"],
        mac.records
    );
}

#[test]
fn filenames_singleline_records() {
    let handler = Handler {
        pattern: RegexSet::new(&[r"o"]).unwrap(),
        log_pattern: Regex::new(r".").unwrap(),
        filename: true,
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
}

#[test]
fn filenames_multiline_records() {
    let handler = Handler {
        pattern: RegexSet::new(&[r"r"]).unwrap(),
        log_pattern: Regex::new(r"e").unwrap(),
        filename: true,
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
}

#[test]
fn filenames_final_newline() {
    let handler = Handler {
        pattern: RegexSet::new(&[r"r"]).unwrap(),
        log_pattern: Regex::new(r"e").unwrap(),
        filename: true,
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
}

#[test]
fn max_count() {
    let handler = Handler {
        pattern: RegexSet::new(&[r"t", r"u"]).unwrap(),
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
