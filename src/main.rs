use std::process::ExitCode;

use lgrep::Exit;

fn main() -> ExitCode {
    match lgrep::run() {
        Err(e) => {
            eprintln!("lgrep: {e:#}");
            Exit::Error.into()
        }
        Ok(e) => e.into(),
    }
}
