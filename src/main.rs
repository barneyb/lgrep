use std::process;

fn main() {
    match lgrep::run() {
        Err(e) => {
            eprintln!("Application error: {e}");
            process::exit(2);
        }
        Ok(-1) => process::exit(2),
        Ok(matches) => process::exit(if matches == 0 { 1 } else { 0 }),
    }
}
