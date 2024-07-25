use std::io::BufRead;

use anyhow::{Context, Result};

pub(crate) const STDIN_FILENAME: &str = "-";

pub(crate) mod lines;
pub(crate) mod records;
pub(crate) mod source;

/// Open a [BufRead] for the named file, or STDIN if the filename is '-'. If the
/// stream is compressed using a well-known format (e.g. gzip), it will be
/// decompressed automatically _on Unix-ish platforms_, by shelling out to an
/// appropriate utility on your `$PATH`. On Windows, you must manually
/// decompress the stream/file first.
pub(crate) fn get_reader(filename: &String) -> Result<Box<dyn BufRead>> {
    if filename == STDIN_FILENAME {
        open_stdin().with_context(|| "Failed to open STDIN for reading")
    } else {
        open_file(filename).with_context(|| format!("Failed to open '{filename}' for reading"))
    }
}

#[cfg(not(target_os = "windows"))]
fn open_stdin() -> Result<Box<dyn BufRead>> {
    use compress_io::compress::CompressIo;
    Ok(Box::new(CompressIo::new().bufreader()?))
}

#[cfg(not(target_os = "windows"))]
fn open_file(filename: &String) -> Result<Box<dyn BufRead>> {
    use compress_io::compress::CompressIo;
    Ok(Box::new(CompressIo::new().path(filename).bufreader()?))
}

#[cfg(target_os = "windows")]
fn open_stdin() -> Result<Box<dyn BufRead>> {
    use std::io::stdin;
    Ok(Box::new(stdin().lock()))
}

#[cfg(target_os = "windows")]
fn open_file(filename: &String) -> Result<Box<dyn BufRead>> {
    use std::fs::File;
    use std::io::BufReader;
    Ok(Box::new(BufReader::new(File::open(filename)?)))
}
