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
    pub fn open(path: &Path, initialize: bool) -> Result<Self> {
        // 1️⃣ Ensure file + header exist
        let initializer = Initializer::new(path);
        if initialize {
            initializer.init_db_file(); // safe to call multiple times
        }

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
        let catalog = match pager.load_catalog() {
            Ok(catalog) => catalog,
            Err(e) => {
                // 👇 decide what “do something” means
                // Option A: initialize a new catalog
                pager.init_catalog_root()?;
                pager.load_catalog()?
            }
        };
        // catalog.validate(); // optional but recommended

        let mut db =Self {
            pager,
            catalog,
        };


        if(initialize){
            db.seed_schema().unwrap();
        }

        Ok(db)
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
    pub fn add_column(
        &mut self,
        table_name: &str,
        column: crate::records::table_column::TableColumn,
    ) -> Result<()> {
        let table_id = *self.catalog.tables_by_name
            .get(table_name)
            .ok_or_else(|| Error::new(std::io::ErrorKind::NotFound, "table not found"))?;
    
        let col = self.pager.add_column(table_name, column)?;
    
        self.catalog
            .columns_by_table
            .entry(table_id)
            .or_default()
            .push(col);
    
        Ok(())
    }

    fn seed_schema(&mut self) -> std::io::Result<()> {
        use crate::records::table_column::TableColumn;

        let tables = [
            ("users", vec![
                ("id"),
                ("first_name"),
                ("last_name"),
                ("email"),
                ("password_hash"),
                ("dob"),
                ("is_active"),
                ("created_at"),
                ("updated_at"),
                ("deleted_at"),
            ]),
            ("orders", vec![
                ("id"),
                ("user_id"),
                ("status"),
                ("subtotal"),
                ("tax"),
                ("total"),
                ("currency"),
                ("created_at"),
                ("updated_at"),
                ("deleted_at"),
            ]),
            ("products", vec![
                ("id"),
                ("sku"),
                ("name"),
                ("description"),
                ("price"),
                ("stock"),
                ("category_id"),
                ("is_active"),
                ("created_at"),
                ("updated_at"),
            ]),
            ("categories", vec![
                ("id"),
                ("name"),
                ("slug"),
                ("parent_id"),
                ("sort_order"),
                ("is_active"),
                ("created_at"),
                ("updated_at"),
                ("deleted_at"),
                ("metadata"),
            ]),
            ("payments", vec![
                ("id"),
                ("order_id"),
                ("provider"),
                ("provider_ref"),
                ("amount"),
                ("currency"),
                ("status"),
                ("paid_at"),
                ("created_at"),
                ("updated_at"),
            ]),
            ("addresses", vec![
                ("id"),
                ("user_id"),
                ("line1"),
                ("line2"),
                ("city"),
                ("country"),
                ("postal_code"),
                ("is_default"),
                ("created_at"),
                ("updated_at"),
            ]),
            ("sessions", vec![
                ("id"),
                ("user_id"),
                ("token"),
                ("ip_address"),
                ("user_agent"),
                ("expires_at"),
                ("revoked_at"),
                ("created_at"),
                ("updated_at"),
                ("last_seen_at"),
            ]),
            ("roles", vec![
                ("id"),
                ("name"),
                ("description"),
                ("is_system"),
                ("created_at"),
                ("updated_at"),
                ("deleted_at"),
                ("permissions"),
                ("priority"),
                ("metadata"),
            ]),
            ("user_roles", vec![
                ("id"),
                ("user_id"),
                ("role_id"),
                ("assigned_by"),
                ("assigned_at"),
                ("expires_at"),
                ("is_active"),
                ("created_at"),
                ("updated_at"),
                ("deleted_at"),
            ]),
            ("audit_logs", vec![
                ("id"),
                ("actor_id"),
                ("action"),
                ("entity"),
                ("entity_id"),
                ("payload"),
                ("ip_address"),
                ("created_at"),
                ("request_id"),
                ("severity"),
            ]),
        ];

        for (table_name, columns) in tables {
            if !self.catalog.tables_by_name.contains_key(table_name) {
                self.create_table(table_name)?;
            }

            for (col_name) in columns {
                self.add_column(
                    table_name,
                    TableColumn {
                        column_id: 0, // assigned internally
                        table_id: 0,  // resolved internally
                        name: col_name.to_string()
                    },
                )?;
            }
        }

        Ok(())
    }
}
