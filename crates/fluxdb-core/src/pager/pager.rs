use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Error, Read, Seek, SeekFrom, Write};
use crate::general::catalog::Catalog;
use crate::general::header::Header;
use crate::pager::page::Page;
use crate::pager::page_type::PageType;
use crate::records::catalog_root::CatalogRoot;
use crate::records::db_record::DbRecord;
use crate::records::record::Record;
use crate::records::record_type::RecordType;
use crate::records::table_column::TableColumn;
use crate::records::table_meta::TableMeta;


pub struct Pager {
    pub header: Header,
    file: RefCell<File>,
}

impl Pager {
    pub fn new(file: File, header: Header) -> Self {
        Self { file: RefCell::new(file), header }
    }

    pub fn page_offset(&self, page_id: u64) -> u64 {
        Header::SIZE as u64 + page_id * self.header.page_size as u64
    }

    pub fn allocate_page(&mut self, page_type: PageType) -> Result<Page, Error> {
        let page_id = self.header.page_count; // 0-based page ids
        let offset = self.page_offset(page_id);
        let page_size = self.header.page_size as usize;

        let page = Page::new(page_size, page_type, page_id as u32);

        // ✅ scope the file borrow so it DROPS before flush_header()
        {
            let mut file = self.file.borrow_mut();
            file.seek(SeekFrom::Start(offset))?;
            file.write_all(&page.buf)?;
            file.flush()?;
        } // 👈 borrow released here

        self.header.page_count += 1;
        self.flush_header()?; // ✅ now safe

        Ok(page)
    }

    pub fn read_page(&self, page_id: u64) -> Result<Page, Error> {
        let offset = self.page_offset(page_id);
        let page_size = self.header.page_size as usize;

        let mut file = self.file.borrow_mut();
        file.seek(SeekFrom::Start(offset))?;

        let mut buf = vec![0u8; page_size];
        file.read_exact(&mut buf)?;

        Ok(Page::from_buffer(buf))
    }

    pub fn write_page(&mut self, page_id: u64, page: &Page) -> Result<(), Error> {
        let offset = self.page_offset(page_id);
        let mut file = self.file.borrow_mut();
        file.seek(SeekFrom::Start(offset))?;
        file.write_all(&page.buf)?;
        file.flush()?;
        Ok(())
    }

    pub fn flush_header(&self) -> Result<(), Error> {
        let mut file = self.file.borrow_mut();
        file.seek(SeekFrom::Start(0))?;
        self.header.write_to(&mut *file)?;
        file.flush()?;
        Ok(())
    }

    pub fn insert_record(&mut self, page_id: u64, record: &[u8]) -> Result<u16, Error> {
        let mut page = self.read_page(page_id)?;
        let slot_id = page.insert_record(record)?;
        self.write_page(page_id, &page)?;
        Ok(slot_id)
    }

    pub fn insert_typed<T: DbRecord>(&mut self, page_id: u64, value: &T) -> Result<u16, Error> {
        let mut page = self.read_page(page_id)?;
        let slot_id = page.insert_typed_record(value)?;
        self.write_page(page_id, &page)?;
        Ok(slot_id)
    }

    /// Initializes the catalog layout for a brand-new DB file.
    ///
    /// Storage invariants after success:
    /// - Page 0 is reserved for CatalogRoot and contains ONLY slot 0 = CatalogRoot
    /// - CatalogRoot.catalog_root_page_id points to the FIRST catalog heap page (>= 1)
    /// - Catalog heap pages may be chained via `next_page_id` (0 means end)
    pub fn init_catalog_root(&mut self) -> Result<(), Error> {
        // 1) Allocate page 0: CatalogRoot page (reserved, never used as a heap)
        let root_page = self.allocate_page(PageType::CatalogPage)?;
        assert_eq!(
            root_page.header.page_id, 0,
            "CatalogRoot must live on page 0 (reserved)"
        );

        // 2) Allocate page 1: the first Catalog HEAP page (where TableMeta/ColumnMeta live)
        let catalog_heap_root = self.allocate_page(PageType::CatalogPage)?;
        assert_ne!(
            catalog_heap_root.header.page_id, 0,
            "Catalog heap root must not be page 0"
        );

        // 3) Create CatalogRoot pointing at the catalog heap root page
        let catalog_root = CatalogRoot {
            version: 1,
            next_table_id: 1,
            next_column_id: 1,
            catalog_root_page_id: catalog_heap_root.header.page_id, // heap pointer
        };

        // 4) Write a fresh page 0 containing ONLY CatalogRoot in slot 0
        //    (We rebuild the page rather than "update slot 0" to avoid needing an update API.)
        let page_size = self.header.page_size as usize;
        let mut fresh_root_page = Page::new(page_size, PageType::CatalogPage, 0);
        fresh_root_page
            .insert_typed_record(&catalog_root)
            .map_err(|e| Error::new(e.kind(), format!("failed to insert CatalogRoot: {e}")))?;

        self.write_page(0, &fresh_root_page)?;

        Ok(())
    }

    pub fn load_catalog_root(&mut self) -> Result<CatalogRoot, Error> {
        let page0 = self.read_page(0)?;

        let raw = page0.read_record(0).unwrap();

        let (record_type, payload) = Record::decode(raw).unwrap();

        if record_type != RecordType::CatalogRoot {
            return Err(Error::new(
                std::io::ErrorKind::InvalidData,
                "slot 0 on page 0 is not a CatalogRoot record",
            ));
        }

        let catalog_root = CatalogRoot::deserialize(payload)
            .map_err(|e| Error::new(std::io::ErrorKind::InvalidData, e))?;

        Ok(catalog_root)
    }

    /// Loads and prints tables from the CATALOG HEAP (not page 0).
    /// Traverses from CatalogRoot.catalog_root_page_id following next_page_id (0 means end).
    pub fn load_db_tables(&mut self) -> Result<(), Error> {

        let mut tables: Vec<TableMeta> = Vec::new();
        let mut cols: Vec<TableColumn> = Vec::new();

        let root = self.load_catalog_root()?;

        // catalog_root_page_id must point to the FIRST catalog heap page (>= 1)
        let mut page_id = root.catalog_root_page_id as u64;
        if page_id == 0 {
            return Err(Error::new(
                std::io::ErrorKind::InvalidData,
                "CatalogRoot.catalog_root_page_id is 0 (invalid). Catalog heap root must be >= 1.",
            ));
        }

        while page_id != 0 {
            let page = self.read_page(page_id)?;

            let slot_count = page.header.slot_count;
            for i in 0..slot_count {
                let raw = page.read_record(i).unwrap();

                let (record_type, payload) = Record::decode(raw).unwrap();

                match record_type {
                    RecordType::CatalogTable => {
                        let table = TableMeta::deserialize(payload)
                            .map_err(|e| Error::new(std::io::ErrorKind::InvalidData, e))?;
                        tables.push(table);
                    }
                    RecordType::CatalogRoot => {
                        // CatalogRoot should live only on page 0; seeing it in heap is suspicious.
                        return Err(Error::new(
                            std::io::ErrorKind::InvalidData,
                            "found CatalogRoot record inside catalog heap (unexpected).",
                        ));
                        print!("Found Catalog Column");

                    }
                    RecordType::CatalogColumn => {
                        let column = TableColumn::deserialize(payload)
                            .map_err(|e| Error::new(std::io::ErrorKind::InvalidData, e))?;
                        cols.push(column);
                    }
                    _ => {
                        // In the future you’ll add ColumnMeta, IndexMeta, etc.
                        // For now, just ignore unknown record types rather than panic.
                    }
                }
            }

            // Convention: next_page_id = 0 means end-of-chain (safe because page 0 is reserved)
            let next = page.header.next_page_id;
            page_id = next as u64;
        }

        for table in &tables {
            println!("==============================");
            println!("Table: {} (id={})", table.name, table.table_id);

            let table_columns: Vec<_> = cols
                .iter()
                .filter(|c| c.table_id == table.table_id)
                .collect();

            if table_columns.is_empty() {
                println!("  (no columns)");
                continue;
            }

            for col in table_columns {
                println!(
                    "  - Column: {} (id={})",
                    col.name,
                    col.column_id,
                );
            }
        }

        Ok(())
    }

    pub fn load_catalog(&mut self) -> Result<Catalog, Error> {
        let mut tables: Vec<TableMeta> = Vec::new();
        let mut cols: Vec<TableColumn> = Vec::new();

        let root = self.load_catalog_root()?;

        let mut page_id = root.catalog_root_page_id as u64;
        if page_id == 0 {
            return Err(Error::new(
                std::io::ErrorKind::InvalidData,
                "CatalogRoot.catalog_root_page_id is 0 (invalid)",
            ));
        }

        while page_id != 0 {
            let page = self.read_page(page_id)?;

            for slot in 0..page.header.slot_count {
                let raw = page.read_record(slot).unwrap();

                let (record_type, payload) = Record::decode(raw).unwrap();

                match record_type {
                    RecordType::CatalogTable => {
                        let table = TableMeta::deserialize(payload)
                            .map_err(|e| Error::new(std::io::ErrorKind::InvalidData, e))?;
                        tables.push(table);
                    }
                    RecordType::CatalogColumn => {
                        let column = TableColumn::deserialize(payload)
                            .map_err(|e| Error::new(std::io::ErrorKind::InvalidData, e))?;
                        cols.push(column);
                    }
                    RecordType::CatalogRoot => {
                        return Err(Error::new(
                            std::io::ErrorKind::InvalidData,
                            "CatalogRoot found inside catalog heap",
                        ));
                    }
                    _ => {}
                }
            }

            page_id = page.header.next_page_id as u64;
        }

        // Build indexes
        let mut tables_by_id = HashMap::new();
        let mut tables_by_name = HashMap::new();
        let mut columns_by_table: HashMap<u32, Vec<TableColumn>> = HashMap::new();

        for table in tables {
            tables_by_name.insert(table.name.clone(), table.table_id);
            tables_by_id.insert(table.table_id, table);
        }

        for col in cols {
            columns_by_table
                .entry(col.table_id)
                .or_default()
                .push(col);
        }

        Ok(Catalog {
            tables_by_id,
            tables_by_name,
            columns_by_table,
        })
    }

    pub fn create_table(&mut self, table_name: &str) -> Result<TableMeta, Error> {
        // 1) Load CatalogRoot (page 0)
        let mut root = self.load_catalog_root()?;

        let table_id = root.next_table_id;

        let table_meta = TableMeta {
            table_id,
            name: table_name.to_string(),
        };

        // 2) Walk catalog heap pages to find space
        let mut page_id = root.catalog_root_page_id as u64;

        loop {
            let mut page = self.read_page(page_id)?;

            // Try inserting into this page
            match page.insert_typed_record(&table_meta) {
                Ok(_slot_id) => {
                    // Success → persist page
                    self.write_page(page_id, &page)?;
                    break;
                }
                Err(_) => {
                    // Page full → follow or create next
                    if page.header.next_page_id != 0 {
                        page_id = page.header.next_page_id as u64;
                    } else {
                        // Allocate a new catalog heap page
                        let new_page = self.allocate_page(PageType::CatalogPage)?;
                        let new_page_id = new_page.header.page_id;

                        // Link pages
                        page.header.next_page_id = new_page_id;
                        self.write_page(page_id, &page)?;

                        page_id = new_page_id as u64;
                    }
                }
            }
        }

        // 3) Update CatalogRoot.next_table_id and persist it
        root.next_table_id += 1;
        self.persist_catalog_root(&root)?;

        Ok(table_meta)
    }

    pub fn find_table_by_name(
        &mut self,
        table_name: &str,
    ) -> Result<TableMeta, Error> {
        let root = self.load_catalog_root()?;
        let mut page_id = root.catalog_root_page_id as u64;

        while page_id != 0 {
            let page = self.read_page(page_id)?;
            for slot in 0..page.header.slot_count {
                let raw = page.read_record(slot).unwrap();
                let (ty, payload) = Record::decode(raw).unwrap();

                if ty == RecordType::CatalogTable {
                    let table = TableMeta::deserialize(payload)
                        .map_err(|e| Error::new(std::io::ErrorKind::InvalidData, e))?;

                    if table.name == table_name {
                        return Ok(table);
                    }
                }
            }
            page_id = page.header.next_page_id as u64;
        }

        Err(Error::new(
            std::io::ErrorKind::NotFound,
            format!("table '{table_name}' not found"),
        ))
    }

    pub fn add_column(
        &mut self,
        table_name: &str,
        data_type: TableColumn,
    ) -> Result<TableColumn, Error> {
        // 1) Resolve table name → table_id
        let table = self.find_table_by_name(table_name)?;

        // 2) Load & increment CatalogRoot
        let mut root = self.load_catalog_root()?;
        let column_id = root.next_column_id;

        let column = TableColumn {
            column_id,
            table_id: table.table_id,
            name: data_type.name,
        };

        // 3) Insert ColumnMeta into catalog heap
        let mut page_id = root.catalog_root_page_id as u64;

        loop {
            let mut page = self.read_page(page_id)?;

            match page.insert_typed_record(&column) {
                Ok(_) => {
                    self.write_page(page_id, &page)?;
                    break;
                }
                Err(_) => {
                    if page.header.next_page_id != 0 {
                        page_id = page.header.next_page_id as u64;
                    } else {
                        let new_page = self.allocate_page(PageType::CatalogPage)?;
                        page.header.next_page_id = new_page.header.page_id;
                        self.write_page(page_id, &page)?;
                        page_id = new_page.header.page_id as u64;
                    }
                }
            }
        }

        // 4) Persist updated CatalogRoot
        root.next_column_id += 1;
        self.persist_catalog_root(&root)?;

        Ok(column)
    }

    fn persist_catalog_root(&mut self, root: &CatalogRoot) -> Result<(), Error> {
        let page_size = self.header.page_size as usize;

        let mut page0 = Page::new(page_size, PageType::CatalogPage, 0);
        page0.insert_typed_record(root)
            .map_err(|e| Error::new(std::io::ErrorKind::Other, e))?;

        self.write_page(0, &page0)?;
        Ok(())
    }
}
