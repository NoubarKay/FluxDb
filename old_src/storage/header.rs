use std::io;
use std::io::{Read, Seek, SeekFrom, Write};
use crate::storage::error::FluxError;

pub const DB_MAGIC: [u8; 16] = *b"FLUXDB_FAST\0\0\0\0\0";
pub const DB_HEADER_SIZE: u16 = 128;
pub const DB_VERSION: u32 = 1;

// const _: () = assert!(size_of::<FluxDbFileHeader>() == 128);


// FluxDbFileHeader is the first page of every FluxDB file. It contains
// metadata about the database file format and global state.
//
#[repr(C)]
#[derive(Debug, Clone)]
pub struct FluxDbFileHeader{
    pub magic: [u8; 16],
    pub header_size: u16,
    pub page_size: u16,
    pub db_version: u32,
    pub write_version: u8,
    pub read_version: u8,
    pub flags: u16,
    pub created_at: u64,
    pub page_count: u64,
    pub checksum: u32,
    pub reserved: [u8; 80]
}


impl FluxDbFileHeader {
    pub fn new(page_size: u16) -> Self{
        Self{
            magic: DB_MAGIC,
            header_size: DB_HEADER_SIZE,
            page_size,
            db_version: DB_VERSION,
            write_version: 1,
            read_version: 1,
            flags: 0,
            created_at: current_unix_time(),
            page_count: 0,
            checksum: 0,
            reserved: [0; 80]
        }
    }

    pub fn write_to<W: Write + Seek>(&self, writer: &mut W) -> FluxError::Result<()> {
        writer.seek(SeekFrom::Start(0))?;

        writer.write_all(&self.magic)?;
        writer.write_all(&self.header_size.to_le_bytes())?;
        writer.write_all(&self.page_size.to_le_bytes())?;
        writer.write_all(&self.db_version.to_le_bytes())?;
        writer.write_all(&[self.write_version])?;
        writer.write_all(&[self.read_version])?;
        writer.write_all(&self.flags.to_le_bytes())?;
        writer.write_all(&self.created_at.to_le_bytes())?;
        writer.write_all(&self.page_count.to_le_bytes())?;
        writer.write_all(&self.checksum.to_le_bytes())?;
        writer.write_all(&self.reserved)?;

        writer.flush()?;
        Ok(())
    }

    pub fn read_from<R: Read + Seek>(reader: &mut R) -> FluxError::Result<Self> {
        reader.seek(SeekFrom::Start(0))?;

        let mut magic = [0u8; 16];
        reader.read_exact(&mut magic)?;

        if magic != DB_MAGIC {
            return Err(FluxError::FluxError::CorruptData("Invalid DB magic header"));
        }

        let header_size = read_u16(reader)?;
        if header_size != DB_HEADER_SIZE {
            return Err(FluxError::FluxError::CorruptData("Unsupported header size"));
        }

        Ok(Self {
            magic,
            header_size,
            page_size: read_u16(reader)?,
            db_version: read_u32(reader)?,
            write_version: read_u8(reader)?,
            read_version: read_u8(reader)?,
            flags: read_u16(reader)?,
            created_at: read_u64(reader)?,
            page_count: read_u64(reader)?,
            checksum: read_u32(reader)?,
            reserved: {
                let mut r = [0u8; 80];
                reader.read_exact(&mut r)?;
                r
            },
        })
    }
}

fn current_unix_time() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn read_u8<R: Read>(r: &mut R) -> FluxError::Result<u8> {
    let mut b = [0u8; 1];
    r.read_exact(&mut b)?;
    Ok(b[0])
}

fn read_u16<R: Read>(r: &mut R) -> FluxError::Result<u16> {
    let mut b = [0u8; 2];
    r.read_exact(&mut b)?;
    Ok(u16::from_le_bytes(b))
}

fn read_u32<R: Read>(r: &mut R) -> FluxError::Result<u32> {
    let mut b = [0u8; 4];
    r.read_exact(&mut b)?;
    Ok(u32::from_le_bytes(b))
}

fn read_u64<R: Read>(r: &mut R) -> FluxError::Result<u64> {
    let mut b = [0u8; 8];
    r.read_exact(&mut b)?;
    Ok(u64::from_le_bytes(b))
}