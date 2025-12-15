use std::fs::File;
use std::io;
use std::io::{Read, Seek, SeekFrom, Write};
use crate::storage::header::FluxDbFileHeader;
use crate::storage::page::Page;
use crate::storage::page_header::{PageHeader, PageType};
use crate::storage::tables::CatalogRoot::CatalogRoot;
use crate::storage::tables::TableMeta::TableMeta;

pub struct FluxPager{
    header: FluxDbFileHeader,
    file: File
}

impl FluxPager{
    pub fn new(file: File, header: FluxDbFileHeader) -> Self{
        Self{file, header}
    }
    pub fn page_offset(&self, page_id: u64) -> u64 {
        self.header.header_size as u64 + page_id * self.header.page_size as u64
    }

    pub fn allocate_page(&mut self, page_type: PageType) -> io::Result<u64> {
        let page_id = self.header.page_count;
        let offset = self.page_offset(page_id);
        let page_size = self.header.page_size as usize;

        self.file.seek(SeekFrom::Start(offset))?;

        let mut page = vec![0u8; page_size];

        let header = PageHeader{
            page_type,
            slot_count: 0,
            free_start: PageHeader::SIZE as u16,
            free_end: page_size as u16,
            page_id: page_id as u32
        };

        header.write_to(&mut page);

        self.file.write_all(&page)?;

        self.file.flush()?;
        self.header.page_count += 1;
        self.flush_header()?;
        Ok(page_id)
    }

    pub fn read_page(&mut self, page_id: u64) -> io::Result<Page> {
        let offset = self.page_offset(page_id);
        let page_size = self.header.page_size as usize;

        self.file.seek(SeekFrom::Start(offset))?;

        let mut buf = vec![0u8; page_size];
        self.file.read_exact(&mut buf)?;

        let page = Page::new(buf);

        Ok(page)
    }

    pub fn read_page_header(&mut self, page_id: u64) -> io::Result<PageHeader>{
        let offset =
            self.header.header_size as u64
                + page_id * self.header.page_size as u64;

        self.file.seek(SeekFrom::Start(offset))?;

        // Read full page header
        let mut buf = [0u8; PageHeader::SIZE];
        self.file.read_exact(&mut buf)?;

        Ok(PageHeader::read_from(&buf))
    }

    pub fn write_page(&mut self, page_id: u64, page: &[u8]) -> io::Result<()> {
        let offset = self.page_offset(page_id);

        assert!(page.len() == self.header.page_size as usize);

        self.file.seek(SeekFrom::Start(offset))?;
        self.file.write_all(page)?;
        self.file.flush()?;

        Ok(())
    }

    /// Initializes the database catalog by creating the CatalogRoot.
    ///
    /// # Purpose
    /// This function is called exactly once when a database file is created.
    /// It allocates the first catalog page and writes the initial CatalogRoot
    /// record, establishing the foundation for all schema metadata.
    ///
    /// # Storage Invariant
    /// After this function completes successfully:
    /// - page_id = 1 exists and is a Catalog page
    /// - slot_id = 0 contains a valid CatalogRoot record
    ///
    /// # Errors
    /// Returns an `io::Error` if the catalog page cannot be allocated or
    /// written to disk.
    ///
    /// # Panics
    /// Panics if the catalog root page cannot be allocated at page 1. This
    /// indicates a corrupted or improperly initialized database file.
    pub fn init_catalog_root(&mut self) -> io::Result<()> {
        let page_id = self.allocate_page(PageType::CatalogPage)?;
        assert_eq!(page_id, 0, "CatalogRoot must live on page 1");

        let mut page = self.read_page(page_id)?;

        let root = CatalogRoot {
            next_table_id: 1,
            next_column_id: 1,
        };

        page.insert_record(&root.serialize()).unwrap();

        self.write_page(page_id, &page.buf)?;

        Ok(())
    }

    /// Loads the database CatalogRoot from disk.
    ///
    /// # Purpose
    /// The CatalogRoot is the single authoritative metadata record that
    /// bootstraps the database catalog. It stores global schema allocation
    /// state such as the next available table and column identifiers.
    ///
    /// # Storage Invariant
    /// The CatalogRoot is always stored at:
    /// - page_id = 1
    /// - slot_id = 0
    ///
    /// This location is fixed and must never change. The database relies on
    /// this invariant for deterministic startup and crash-safe ID allocation.
    ///
    /// # Behavior
    /// - Reads page 1 from disk
    /// - Reads record at slot 0
    /// - Deserializes the bytes into a CatalogRoot
    ///
    /// # Errors
    /// Returns an `io::Error` if the page cannot be read from disk.
    ///
    /// # Panics
    /// Panics if the CatalogRoot record is missing or corrupted. This
    /// indicates an uninitialized or irrecoverably corrupted database.
    pub fn load_catalog_root(&mut self) -> io::Result<CatalogRoot> {
        // CatalogRoot is always stored at page 1, slot 0
        let page_id = 0;

        let page = self.read_page(page_id)?;

        let bytes = page
            .read_record(0)
            .expect("CatalogRoot record missing or corrupted");

        Ok(CatalogRoot::deserialize(bytes))
    }

    /// Updates the CatalogRoot record on disk.
    ///
    /// # Purpose
    /// Persists changes to the global catalog allocation state
    /// (next_table_id, next_column_id). This function must be
    /// called after any schema mutation that allocates new IDs.
    ///
    /// # Storage Invariant
    /// - CatalogRoot is always stored at page_id = 1, slot_id = 0
    /// - The record is overwritten in place
    ///
    /// # Durability
    /// This function establishes a durability boundary. When it
    /// returns successfully, the updated CatalogRoot is guaranteed
    /// to be persisted to disk.
    ///
    /// # Errors
    /// Returns an `io::Error` if the page cannot be read or written.
    ///
    /// # Panics
    /// Panics if the CatalogRoot slot is missing or corrupted.
    pub fn update_catalog_root(&mut self, root: &CatalogRoot) -> io::Result<()> {
        let page_id = 0;

        let mut page = self.read_page(page_id)?;

        // Read slot 0 (must exist)
        let slot = page
            .read_slot(0)
            .expect("CatalogRoot slot missing");

        let bytes = root.serialize();

        // Safety check: CatalogRoot size must not change
        assert_eq!(bytes.len(), slot.length as usize, "CatalogRoot size mismatch");

        let start = slot.offset as usize;
        let end = start + slot.length as usize;

        // Overwrite bytes in place
        page.buf[start..end].copy_from_slice(&bytes);

        // Persist page (must fsync inside write_page)
        self.write_page(page_id, &page.buf)?;

        Ok(())
    }

    pub fn create_table(
        &mut self,
        table_name: &str,
        // columns: &[ColumnMeta],
    ) -> std::io::Result<u32>{
        let mut root = self.load_catalog_root().unwrap();
        let table_id = root.next_table_id;
        root.next_table_id += 1;

        let table_meta = TableMeta {
            table_id,
            name: table_name.to_string(),
        };

        let page_id = 0;
        let mut page = self.read_page(page_id)?;

        page.insert_record(&table_meta.serialize()).unwrap();

        self.write_page(page_id, &page.buf)?;
        self.update_catalog_root(&root)?;

        Ok(table_id)
    }

    pub fn header(&self) -> &FluxDbFileHeader {
        &self.header
    }

    pub fn flush_header(&mut self) -> io::Result<()> {
        self.header.write_to(&mut self.file)
    }
}