use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use crate::general::header::Header;
use crate::helpers::header_flags::HeaderFlags;

pub struct Initializer {
    pub path: PathBuf,
}

impl Initializer {
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
        }
    }

    pub fn init_db_file(&self) {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.path).unwrap();

        let flags = HeaderFlags::CHECKSUM_ENABLED | HeaderFlags::COLUMNAR_V1 | HeaderFlags::COMPRESSION;

        let mut header = Header::new(4096, flags);
        header.write_to(&mut file).unwrap();
    }


    pub fn read_header(&self) -> Header {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&self.path).unwrap();

        let header = Header::read_from(&mut file).unwrap();
        header
    }
}