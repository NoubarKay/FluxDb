use std::fs::{File, OpenOptions};
use std::path::Path;
use crate::storage::error::FluxError;


pub fn create_db_file(path: &Path) -> FluxError::Result<File> {
    Ok(OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?)
}

pub fn open_db_file(path: &Path) -> FluxError::Result<File> {
    Ok(OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)?)
}