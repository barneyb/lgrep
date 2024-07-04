use std::process;

fn main() {
    if let Err(e) = lgrep::run() {
        println!("Application error: {e}");
        process::exit(1);
    }
}
