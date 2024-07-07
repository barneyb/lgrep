use std::io::BufRead;

use anyhow::Context;
use compress_io::compress::CompressIo;

pub(crate) const STD_IN_FILENAME: &str = "-";

pub fn get_reader(filename: &String) -> anyhow::Result<Box<dyn BufRead>> {
    Ok(if filename == STD_IN_FILENAME {
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
