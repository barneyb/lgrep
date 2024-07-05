use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use anyhow::Context;

use crate::cli::STD_IN_FILENAME;

pub fn get_reader(filename: &String) -> anyhow::Result<Box<dyn BufRead>> {
    if filename == STD_IN_FILENAME {
        Ok(Box::new(std::io::stdin().lock()))
    } else {
        Ok(Box::new(BufReader::new(
            File::open(filename)
                .with_context(|| format!("Failed to open '{filename}' for reading"))?,
        )))
    }
}
