use std::io::BufRead;

use anyhow::{Context, Result};
use compress_io::compress::CompressIo;

pub(crate) const STDIN_FILENAME: &str = "-";

pub fn get_reader(filename: &String) -> Result<Box<dyn BufRead>> {
    Ok(if filename == STDIN_FILENAME {
        Box::new(
            CompressIo::new() // implicitly STDIN
                .bufreader()
                .with_context(|| "Failed to open STDIN for reading")?,
        )
    } else {
        Box::new(
            CompressIo::new()
                .path(filename)
                .bufreader()
                .with_context(|| format!("Failed to open '{filename}' for reading"))?,
        )
    })
}
