use clap::ColorChoice;

use crate::cli::Cli;
use crate::handler::Handler;

use super::*;

#[test]
fn empty() {
    let h: Handler = Cli::empty().into();
    assert_eq!(vec!["-"], h.files);
    assert!(h.pattern_set.is_empty()); // nonsense, but lib::run guards us
    assert_eq!(None, h.max_count);
    assert!(!h.invert_match);
    assert!(!h.counts);
    assert_eq!(ColorChoice::Auto, h.color_mode);
    assert_eq!(None, h.stdin_label);
    assert_eq!(DEFAULT_LOG_PATTERN, h.log_pattern.to_string());
    assert_eq!(None, h.start.map(|re| re.to_string()));
    assert_eq!(None, h.end.map(|re| re.to_string()));
    assert!(!h.filenames);
}

#[test]
fn pattern() {
    let h: Handler = Cli {
        pattern: Some("goat".to_owned()),
        ..Cli::empty()
    }
    .into();
    assert_eq!(vec!["goat"], h.pattern_set.patterns());
}

#[test]
fn patterns() {
    let h: Handler = Cli {
        patterns: vec![Regex::new("a").unwrap(), Regex::new("b").unwrap()],
        ..Cli::empty()
    }
    .into();
    assert_eq!(vec!["a", "b"], h.pattern_set.patterns());
}

#[test]
fn pattern_and_patterns() {
    let h: Handler = Cli {
        pattern: Some("goat".to_owned()),
        patterns: vec![Regex::new("a").unwrap(), Regex::new("b").unwrap()],
        ..Cli::empty()
    }
    .into();
    assert_eq!(vec!["a", "b", "goat"], h.pattern_set.patterns());
}

#[test]
fn ignore_case() {
    let h: Handler = Cli {
        ignore_case: true,
        ..Cli::all_re()
    }
    .into();
    assert_eq!(vec!["(?i)Q", "(?i)R", "(?i)P"], h.pattern_set.patterns());
    assert_eq!("(?i)L", h.log_pattern.to_string());
    assert_eq!(Some("(?i)S".to_owned()), h.start.map(|re| re.to_string()));
    assert_eq!(Some("(?i)E".to_owned()), h.end.map(|re| re.to_string()));
}

#[test]
fn explicit_stdin() {
    let h: Handler = Cli {
        files: vec!["-".to_owned()],
        ..Cli::empty()
    }
    .into();
    assert_eq!(vec!["-"], h.files);
    assert!(!h.filenames);
}

#[test]
fn one_file() {
    let h: Handler = Cli {
        files: vec!["app.log".to_owned()],
        ..Cli::empty()
    }
    .into();
    assert_eq!(vec!["app.log"], h.files);
    assert!(!h.filenames);
}

#[test]
fn one_file_with_filenames() {
    let h: Handler = Cli {
        files: vec!["app.log".to_owned()],
        filename: true,
        ..Cli::empty()
    }
    .into();
    assert_eq!(vec!["app.log"], h.files);
    assert!(h.filenames);
}

#[test]
fn several_files() {
    let h: Handler = Cli {
        files: vec!["app.log".to_owned(), "-".to_owned(), "cheese".to_owned()],
        ..Cli::empty()
    }
    .into();
    assert_eq!(vec!["app.log", "-", "cheese"], h.files);
    assert!(h.filenames);
}

#[test]
fn several_files_no_filenames() {
    let h: Handler = Cli {
        files: vec!["app.log".to_owned(), "-".to_owned(), "cheese".to_owned()],
        no_filename: true,
        ..Cli::empty()
    }
    .into();
    assert_eq!(vec!["app.log", "-", "cheese"], h.files);
    assert!(!h.filenames);
}

#[test]
fn passthroughs() {
    let h: Handler = Cli {
        max_count: Some(1),
        invert_match: true,
        count: true,
        color: Some(ColorChoice::Always),
        label: Some("goat".to_owned()),
        ..Cli::empty()
    }
    .into();
    assert_eq!(Some(1), h.max_count);
    assert!(h.invert_match);
    assert!(h.counts);
    assert_eq!(ColorChoice::Always, h.color_mode);
    assert_eq!(Some("goat".to_owned()), h.stdin_label);
}
