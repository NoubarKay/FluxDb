use std::fs::OpenOptions;
use std::path::Path;
use std::io::{Error, Result};

use crate::general::catalog::Catalog;
use crate::general::initializer::Initializer;
use crate::pager::pager::Pager;

pub struct Database {
    pub pager: Pager,
    pub catalog: Catalog,
}

impl Database {
    /// Opens an existing database or creates a new one if it does not exist.
    /// Loads the catalog ONCE and caches it in memory.
    pub fn open(path: &Path) -> Result<Self> {
        // 1️⃣ Ensure file + header exist
        let initializer = Initializer::new(path);
        // initializer.init_db_file()?; // safe to call multiple times

        // 2️⃣ Open file
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)?;

        // 3️⃣ Read header
        let header = initializer.read_header();

        // 4️⃣ Create pager
        let mut pager = Pager::new(file, header);

        // 5️⃣ Load catalog ONCE
        let catalog = pager.load_catalog()?;
        // catalog.validate(); // optional but recommended

        Ok(Self {
            pager,
            catalog,
        })
    }

    /// Creates a table (disk + memory)
    pub fn create_table(&mut self, name: &str) -> Result<()> {
        let table = self.pager.create_table(name)?;

        self.catalog
            .tables_by_name
            .insert(table.name.clone(), table.table_id);

        self.catalog
            .tables_by_id
            .insert(table.table_id, table);

        Ok(())
    }

    // Adds a column (disk + memory)
    // pub fn add_column(
    //     &mut self,
    //     table_name: &str,
    //     column: crate::records::table_column::TableColumn,
    // ) -> Result<()> {
    //     let table_id = *self.catalog.tables_by_name
    //         .get(table_name)
    //         .ok_or_else(|| Error::new(std::io::ErrorKind::NotFound, "table not found"))?;
    //
    //     let col = self.pager.add_column(table_id, column)?;
    //
    //     self.catalog
    //         .columns_by_table
    //         .entry(table_id)
    //         .or_default()
    //         .push(col);
    //
    //     Ok(())
    // }
}
