use std::io::{Read, Seek, SeekFrom, Write};
use crc32fast::Hasher;
use crate::helpers::header_flags::HeaderFlags;
use crate::helpers::helper::{current_unix_time, read_u16, read_u32, read_u64, read_u8};

pub const DB_MAGIC: [u8; 16] = *b"FLUXDB_FASTV1\0\0\0";
pub const DB_HEADER_SIZE: u16 = 128;
pub const DB_VERSION: u32 = 1;


#[derive(Debug)]
pub struct Header{
    pub magic: [u8; 16], // 16 BYTES FOR HEADER MAGIC
    pub header_size: u16, // 2 BYTES FOR HEADER SIZE
    pub page_size: u16, // 2 BYTES FOR PAGE SIZE
    pub db_version: u32, // 4 BYTES FOR DB VERSION
    pub write_version: u8, // 1 BYTE FOR WRITE VERSION
    pub read_version: u8, // 1 BYTE FOR READ VERSION
    pub flags: HeaderFlags, // 2 BYTES FOR FLAGS
    pub created_at: u64, // 8 BYTES FOR CREATED AT
    pub page_count: u64, // 8 BYTES FOR PAGE COUNT
    pub checksum: u32, // 4 BYTES FOR CHECKSUM
    pub chunk_catalog_root_page_id: u32, // 4 BYTES FOR CHUNK CATALOG ROOT PAGE ID
    pub reserved: [u8; 76] // 80 BYTES FOR RESERVED
}

impl Header{

    pub const SIZE: usize = 128;

    /// Creates a new database file header (`FluxDbFileHeader`).
    ///
    /// This function initializes a fresh database header intended to be written
    /// at the beginning of a FluxDB data file. The header contains the core
    /// metadata required to identify and validate the file format.
    ///
    /// # Initialized fields
    /// - `magic`: Set to [`DB_MAGIC`], used to verify that the file is a valid FluxDB file.
    /// - `header_size`: Set to [`DB_HEADER_SIZE`], representing the size of the file header in bytes.
    /// - Other fields are initialized to their default / zero values as required
    ///   for a newly created database file.
    ///
    /// # Returns
    /// A fully initialized `FluxDbFileHeader` suitable for writing to disk
    /// during database file creation.
    pub fn new(page_size: u16, flags: HeaderFlags) -> Self {
        Self {
            magic: DB_MAGIC,
            header_size: DB_HEADER_SIZE,
            page_size,
            db_version: DB_VERSION,
            write_version: 1,
            read_version: 1,
            flags,
            created_at: current_unix_time(),   // or unix timestamp later
            page_count: 0,
            checksum: 0,
            chunk_catalog_root_page_id: 0,
            reserved: [0; 76],
        }
    }

    /// Writes the database file header to disk.
    ///
    /// This method serializes the header fields and writes them to the beginning
    /// of the database file. The write always starts at byte offset `0`,
    /// overwriting any existing header data.
    ///
    /// # Behavior
    /// - Seeks to the start of the file (`offset = 0`)
    /// - Writes the header fields in a fixed, little-endian binary layout
    /// - Does **not** flush or sync the underlying writer
    ///
    /// # Disk layout
    /// The fields are written in the following order:
    /// ```text
    /// [ magic (N bytes) | header_size (u16, little-endian) ]
    /// ```
    ///
    /// # Errors
    /// Returns an `io::Error` if seeking or writing to the underlying writer fails.
    ///
    /// # Notes
    /// - This function assumes exclusive access to the file.
    /// - Callers are responsible for ensuring the file is large enough to
    ///   accommodate the header.
    pub fn write_to<W: Write + Seek>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.seek(std::io::SeekFrom::Start(0))?;

        let checksum = self.compute_checksum()?; // ✅ local, derived

        writer.write_all(&self.magic)?;
        writer.write_all(&self.header_size.to_le_bytes())?;
        writer.write_all(&self.page_size.to_le_bytes())?;
        writer.write_all(&self.db_version.to_le_bytes())?;
        writer.write_all(&[self.write_version])?;
        writer.write_all(&[self.read_version])?;
        writer.write_all(&self.flags.bits().to_le_bytes())?;
        writer.write_all(&self.created_at.to_le_bytes())?;
        writer.write_all(&self.page_count.to_le_bytes())?;
        writer.write_all(&checksum.to_le_bytes())?; // ✅ write derived value
        writer.write_all(&self.chunk_catalog_root_page_id.to_le_bytes())?;
        writer.write_all(&self.reserved)?;

        Ok(())
    }

    fn write_without_checksum<W: Write>(&self, mut w: W) -> std::io::Result<()> {
        w.write_all(&self.magic)?;
        w.write_all(&self.header_size.to_le_bytes())?;
        w.write_all(&self.page_size.to_le_bytes())?;
        w.write_all(&self.db_version.to_le_bytes())?;
        w.write_all(&[self.write_version])?;
        w.write_all(&[self.read_version])?;
        w.write_all(&self.flags.bits().to_le_bytes())?;
        w.write_all(&self.created_at.to_le_bytes())?;
        w.write_all(&self.page_count.to_le_bytes())?;
        w.write_all(&self.chunk_catalog_root_page_id.to_le_bytes())?;
        w.write_all(&self.reserved)?;
        Ok(())
    }

    fn compute_checksum(&self) -> std::io::Result<u32> {
        let mut buffer = Vec::with_capacity(DB_HEADER_SIZE as usize);
        self.write_without_checksum(&mut buffer)?;

        let mut hasher = Hasher::new();
        hasher.update(&buffer);
        Ok(hasher.finalize())
    }

    /// Reads and validates the database file header from disk.
    ///
    /// This function reads the header from the beginning of the database file,
    /// deserializes its fields, and performs a basic validation check to ensure
    /// the file is a valid FluxDB data file.
    ///
    /// # Behavior
    /// - Seeks to byte offset `0` before reading
    /// - Reads the header fields in a fixed binary order
    /// - Validates the magic value against [`DB_MAGIC`]
    ///
    /// # Disk layout
    /// The fields are expected in the following order:
    /// ```text
    /// [ magic (16 bytes)         ]
    /// [ header_size (u16 bytes)  ]
    /// [ page_size (u16 bytes)    ]
    /// [ db_version (u32 bytes)   ]
    /// [ write_version (u8 bytes) ]
    /// [ read_version (u8 bytes)  ]
    /// [ flags (u16 bytes)        ]
    /// [ db_version (u64 bytes)   ]
    /// [ created_at (u64 bytes)   ]
    /// [ page_count (u64 bytes)   ]
    /// [ checksum (u32 bytes)     ]
    /// [ reserved (80 bytes)      ]
    ///
    /// ```
    ///
    /// # Errors
    /// Returns an `io::Error` if seeking or reading from the underlying reader fails.
    ///
    /// # Panics
    /// Panics if the magic value does not match [`DB_MAGIC`].
    /// This indicates that the file is not a valid FluxDB database file
    /// or is corrupted.
    ///
    /// # Notes
    /// - No checksum or version validation is performed.
    pub fn read_from<R: Read + Seek>(reader: &mut R) -> std::io::Result<Self> {
        reader.seek(SeekFrom::Start(0))?;

        let mut magic = [0u8; 16];
        reader.read_exact(&mut magic)?;

        if magic != DB_MAGIC{
            //TODO: ADD PROPER ERRORS
            panic!("Invalid DB magic header");
        };

        let header_size = read_u16(reader);
        if header_size != DB_HEADER_SIZE{
            //Todo: ADD PROPER ERRORS
            panic!("Unsupported header size");
        }

        let page_size = read_u16(reader);
        let db_version = read_u32(reader);
        let write_version = read_u8(reader);
        let read_version = read_u8(reader);
        let flags = HeaderFlags::from_bits_truncate(read_u16(reader));
        let created_at = read_u64(reader);
        let page_count = read_u64(reader);
        let checksum = read_u32(reader);
        let chunk_catalog_root_page_id = read_u32(reader);
        let mut reserved = [0u8; 76];
        reader.read_exact(&mut reserved)?;


        let header = Self {
            magic,
            header_size,
            page_size,
            db_version,
            write_version,
            read_version,
            flags,
            created_at,
            page_count,
            checksum,
            chunk_catalog_root_page_id,
            reserved,
        };

        if header.flags.contains(HeaderFlags::CHECKSUM_ENABLED) {
            let computed = header.compute_checksum()?;
            if computed != header.checksum {
                panic!("Header checksum mismatch");
            }
        }

        Ok(header)
    }
}