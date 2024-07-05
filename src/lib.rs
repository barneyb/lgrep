use std::io::{BufRead, Write};

use anyhow::{Context, Result};
use clap::{CommandFactory, Parser};

use cli::Cli;

mod cli;
mod io;

pub fn run() -> Result<()> {
    let args = Cli::parse().init();
    // if no-filename (-h) without any patterns
    if args.no_filename && !args.has_patterns() {
        Cli::command()
            .print_help()
            .with_context(|| "failed to print help")?
    } else if args.help {
        Cli::command()
            .print_long_help()
            .with_context(|| "failed to print long help")?
    } else {
        let mut sink = std::io::stdout().lock();
        for f in args.files.iter() {
            let source = io::get_reader(f)?;
            process_file(&args, source, &mut sink)?
        }
    }
    Ok(())
}

fn process_file(args: &Cli, mut source: Box<dyn BufRead>, sink: &mut dyn Write) -> Result<()> {
    let mut s = String::new();
    while let Ok(n) = source.read_line(&mut s) {
        if n == 0 {
            // reached EOF
            break;
        }
        if let Some(p) = &args.pattern {
            if p.is_match(&s) {
                sink.write_all(s.as_bytes())?;
            }
        }
        s.clear();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn simple_grep() {
        let source = Box::new(Cursor::new(
            b"line one
line two
third line
line 4
",
        ));
        let cli = Cli {
            pattern: Some("t".parse().unwrap()),
            ..Cli::empty()
        };
        let mut sink = Cursor::new(Vec::new());
        process_file(&cli, source, &mut sink).unwrap();
        let bytes = sink.into_inner();
        assert_eq!(
            "line two
third line
",
            String::from_utf8(bytes).unwrap()
        )
    }
}
