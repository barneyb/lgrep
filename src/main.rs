use std::process;

fn main() {
    if let Err(e) = lgrep::run() {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}
