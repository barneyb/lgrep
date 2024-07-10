use std::process;

use lgrep::Exit;

fn main() {
    match lgrep::run() {
        Err(e) => {
            eprintln!("lgrep error: {e}");
            process::exit(2);
        }
        Ok(Exit::Help) => process::exit(2),
        Ok(Exit::Error) => process::exit(2),
        Ok(Exit::Terminate) => process::exit(3),
        Ok(Exit::NoMatch) => process::exit(1),
        Ok(Exit::Match) => process::exit(0),
    }
}
