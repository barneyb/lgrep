use clap::ColorChoice;

use crate::cli::Cli;
use crate::handler::Handler;

use super::*;

#[test]
fn empty() {
    let h = Handler::new(Cli::empty()).unwrap();
    assert_eq!(vec!["-"], h.files);
    assert_eq!(0, h.pattern_set.pattern_len()); // nonsense, but lib::run guards us
    assert_eq!(None, h.max_count);
    assert!(!h.invert_match);
    assert!(!h.counts);
    assert_eq!(ColorChoice::Auto, h.color_mode);
    assert_eq!(None, h.stdin_label);
    assert_re(
        &h.log_pattern,
        &["2024-07-25T12:02:57.123", "0000-00-00 00:00:00.0"],
        &["monday", ""],
    );
    assert!(h.start.is_none());
    assert!(h.end.is_none());
    assert!(!h.filenames);
}

#[test]
fn pattern() {
    let h = Handler::new(Cli {
        pattern: Some("goat".to_owned()),
        ..Cli::empty()
    })
    .unwrap();
    assert_re(&h.pattern_set, &["a goat horn"], &["a cow horn"]);
}

#[test]
fn patterns() {
    let h = Handler::new(Cli {
        patterns: vec!["a".to_owned(), "b".to_owned()],
        ..Cli::empty()
    })
    .unwrap();
    assert_re(&h.pattern_set, &["a", "b"], &["c", "A"]);
}

#[test]
fn pattern_and_patterns() {
    let h = Handler::new(Cli {
        pattern: Some("cow".to_owned()),
        patterns: vec!["a".to_owned(), "b".to_owned()],
        ..Cli::empty()
    })
    .unwrap();
    assert_re(
        &h.pattern_set,
        &["the cowpen", "cantaloupe", "robber"],
        &["cdefghijklmnopqrstuvwxyz", "ABCOW"],
    );
}

#[test]
fn ignore_case() {
    let h = Handler::new(Cli {
        ignore_case: true,
        ..Cli::all_re()
    })
    .unwrap();
    assert_re(
        &h.pattern_set,
        &["q", "r", "p", "Q", "R", "P"],
        &["L", "S", "E"],
    );
    assert_re(&h.log_pattern, &["l", "L"], &["qwertyuiopasdfghjkzxcvbnm"]);
    assert_re(
        &h.start.unwrap(),
        &["s", "S"],
        &["qwertyuiopadfghjklzxcvbnm"],
    );
    assert_re(&h.end.unwrap(), &["e", "E"], &["qwrtyuiopasdfghjklzxcvbnm"]);
}

#[test]
fn explicit_stdin() {
    let h = Handler::new(Cli {
        files: vec!["-".to_owned()],
        ..Cli::empty()
    })
    .unwrap();
    assert_eq!(vec!["-"], h.files);
    assert!(!h.filenames);
}

#[test]
fn one_file() {
    let h = Handler::new(Cli {
        files: vec!["app.log".to_owned()],
        ..Cli::empty()
    })
    .unwrap();
    assert_eq!(vec!["app.log"], h.files);
    assert!(!h.filenames);
}

#[test]
fn one_file_with_filenames() {
    let h = Handler::new(Cli {
        files: vec!["app.log".to_owned()],
        filename: true,
        ..Cli::empty()
    })
    .unwrap();
    assert_eq!(vec!["app.log"], h.files);
    assert!(h.filenames);
}

#[test]
fn several_files() {
    let h = Handler::new(Cli {
        files: vec!["app.log".to_owned(), "-".to_owned(), "cheese".to_owned()],
        ..Cli::empty()
    })
    .unwrap();
    assert_eq!(vec!["app.log", "-", "cheese"], h.files);
    assert!(h.filenames);
}

#[test]
fn several_files_no_filenames() {
    let h = Handler::new(Cli {
        files: vec!["app.log".to_owned(), "-".to_owned(), "cheese".to_owned()],
        no_filename: true,
        ..Cli::empty()
    })
    .unwrap();
    assert_eq!(vec!["app.log", "-", "cheese"], h.files);
    assert!(!h.filenames);
}

#[test]
fn passthroughs() {
    let h = Handler::new(Cli {
        max_count: Some(1),
        invert_match: true,
        count: true,
        color: ColorChoice::Always,
        quiet: true,
        label: Some("goat".to_owned()),
        ..Cli::empty()
    })
    .unwrap();
    assert_eq!(Some(1), h.max_count);
    assert!(h.invert_match);
    assert!(h.counts);
    assert_eq!(ColorChoice::Always, h.color_mode);
    assert!(h.quiet);
    assert_eq!(Some("goat".to_owned()), h.stdin_label);
}
