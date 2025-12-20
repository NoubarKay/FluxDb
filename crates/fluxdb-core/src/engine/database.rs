use std::fs::OpenOptions;
use std::path::Path;
use std::io::{Error, Result};
use crate::engine::catalog::Catalog;
use crate::engine::initializer::Initializer;
use crate::metadata::schema::column_type::ColumnType;
use crate::storage::pager::Pager;

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
        column_name: &str,
        column_type: ColumnType,
    ) -> Result<()> {
        let table_id = *self.catalog.tables_by_name
            .get(table_name)
            .ok_or_else(|| Error::new(std::io::ErrorKind::NotFound, "table not found"))?;
    
        let col = self.pager.add_column(table_name, column_name, column_type)?;
    
        self.catalog
            .columns_by_table
            .entry(table_id)
            .or_default()
            .push(col);
    
        Ok(())
    }

    fn seed_schema(&mut self) -> std::io::Result<()> {

        let tables = [
            ("users", vec![
                ("id", ColumnType::Integer64),
                ("first_name", ColumnType::Utf8),
                ("last_name", ColumnType::Utf8),
                ("email", ColumnType::Utf8),
                ("password_hash", ColumnType::Utf8),
                ("dob", ColumnType::Timestamp),
                ("is_active", ColumnType::Boolean),
                ("created_at", ColumnType::Timestamp),
                ("updated_at", ColumnType::Timestamp),
                ("deleted_at", ColumnType::Timestamp),
            ]),

            ("orders", vec![
                ("id", ColumnType::Integer64),
                ("user_id", ColumnType::Integer64),
                ("status", ColumnType::Utf8),
                ("subtotal", ColumnType::Float64),
                ("tax", ColumnType::Float64),
                ("total", ColumnType::Float64),
                ("currency", ColumnType::Utf8),
                ("created_at", ColumnType::Timestamp),
                ("updated_at", ColumnType::Timestamp),
                ("deleted_at", ColumnType::Timestamp),
            ]),

            ("products", vec![
                ("id", ColumnType::Integer64),
                ("sku", ColumnType::Utf8),
                ("name", ColumnType::Utf8),
                ("description", ColumnType::Utf8),
                ("price", ColumnType::Float64),
                ("stock", ColumnType::Integer32),
                ("category_id", ColumnType::Integer64),
                ("is_active", ColumnType::Boolean),
                ("created_at", ColumnType::Timestamp),
                ("updated_at", ColumnType::Timestamp),
            ]),

            ("categories", vec![
                ("id", ColumnType::Integer64),
                ("name", ColumnType::Utf8),
                ("slug", ColumnType::Utf8),
                ("parent_id", ColumnType::Integer64),
                ("sort_order", ColumnType::Integer32),
                ("is_active", ColumnType::Boolean),
                ("created_at", ColumnType::Timestamp),
                ("updated_at", ColumnType::Timestamp),
                ("deleted_at", ColumnType::Timestamp),
                ("metadata", ColumnType::Utf8),
            ]),

            ("payments", vec![
                ("id", ColumnType::Integer64),
                ("order_id", ColumnType::Integer64),
                ("provider", ColumnType::Utf8),
                ("provider_ref", ColumnType::Utf8),
                ("amount", ColumnType::Float64),
                ("currency", ColumnType::Utf8),
                ("status", ColumnType::Utf8),
                ("paid_at", ColumnType::Timestamp),
                ("created_at", ColumnType::Timestamp),
                ("updated_at", ColumnType::Timestamp),
            ]),

            ("addresses", vec![
                ("id", ColumnType::Integer64),
                ("user_id", ColumnType::Integer64),
                ("line1", ColumnType::Utf8),
                ("line2", ColumnType::Utf8),
                ("city", ColumnType::Utf8),
                ("country", ColumnType::Utf8),
                ("postal_code", ColumnType::Utf8),
                ("is_default", ColumnType::Boolean),
                ("created_at", ColumnType::Timestamp),
                ("updated_at", ColumnType::Timestamp),
            ]),

            ("sessions", vec![
                ("id", ColumnType::Integer64),
                ("user_id", ColumnType::Integer64),
                ("token", ColumnType::Utf8),
                ("ip_address", ColumnType::Utf8),
                ("user_agent", ColumnType::Utf8),
                ("expires_at", ColumnType::Timestamp),
                ("revoked_at", ColumnType::Timestamp),
                ("created_at", ColumnType::Timestamp),
                ("updated_at", ColumnType::Timestamp),
                ("last_seen_at", ColumnType::Timestamp),
            ]),

            ("roles", vec![
                ("id", ColumnType::Integer64),
                ("name", ColumnType::Utf8),
                ("description", ColumnType::Utf8),
                ("is_system", ColumnType::Boolean),
                ("created_at", ColumnType::Timestamp),
                ("updated_at", ColumnType::Timestamp),
                ("deleted_at", ColumnType::Timestamp),
                ("permissions", ColumnType::Utf8),
                ("priority", ColumnType::Integer32),
                ("metadata", ColumnType::Utf8),
            ]),

            ("user_roles", vec![
                ("id", ColumnType::Integer64),
                ("user_id", ColumnType::Integer64),
                ("role_id", ColumnType::Integer64),
                ("assigned_by", ColumnType::Integer64),
                ("assigned_at", ColumnType::Timestamp),
                ("expires_at", ColumnType::Timestamp),
                ("is_active", ColumnType::Boolean),
                ("created_at", ColumnType::Timestamp),
                ("updated_at", ColumnType::Timestamp),
                ("deleted_at", ColumnType::Timestamp),
            ]),

            ("audit_logs", vec![
                ("id", ColumnType::Integer64),
                ("actor_id", ColumnType::Integer64),
                ("action", ColumnType::Utf8),
                ("entity", ColumnType::Utf8),
                ("entity_id", ColumnType::Integer64),
                ("payload", ColumnType::Utf8),
                ("ip_address", ColumnType::Utf8),
                ("created_at", ColumnType::Timestamp),
                ("request_id", ColumnType::Utf8),
                ("severity", ColumnType::Integer32),
            ]),
        ];

        for (table_name, columns) in tables {
            if !self.catalog.tables_by_name.contains_key(table_name) {
                self.create_table(table_name)?;
            }

            for (col_name, col_type) in columns {
                self.add_column(
                    table_name,
                    col_name.into(),
                    col_type
                )?;
            }
        }

        Ok(())
    }
}
