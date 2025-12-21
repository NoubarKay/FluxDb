use std::collections::HashMap;
use std::fs::OpenOptions;
use std::path::Path;
use std::io::{Error, ErrorKind, Result};
use std::ptr::null;
use crate::engine::catalog::Catalog;
use crate::engine::chunk_manager::ChunkManager;
use crate::engine::initializer::Initializer;
use crate::metadata::schema::column_type::ColumnType;
use crate::storage::chunk_data_header::ChunkDataHeader;
use crate::storage::page_header::PageHeader;
use crate::storage::page_type::PageType;
use crate::storage::pager::{PageInit, Pager};

pub struct Database {
    pub catalog: Catalog,
    pub chunk_manager: ChunkManager,
}
impl Database {
    /// Opens an existing database or creates a new one if it does not exist.
    /// Loads the catalog ONCE and caches it in memory.
    pub fn open(path: &Path, initialize: bool) -> Result<Self> {
        let initializer = Initializer::new(path);
        if initialize {
            initializer.init_db_file(); // safe to call multiple times
        }

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)?;

        let header = initializer.read_header();

        let pager = Pager::new(file, header);
        let mut chunk_manager = ChunkManager::new(pager);

        let catalog = match chunk_manager.load_catalog() {
            Ok(catalog) => catalog,
            Err(e) => {
                chunk_manager.init_catalog_root()?;
                chunk_manager.load_catalog()?
            }
        };

        let mut db =Self {
            catalog,
            chunk_manager
        };

        if(initialize){
            db.seed_schema().unwrap();
        }

        Ok(db)
    }

    /// Creates a table (disk + memory)
    pub fn create_table(&mut self, name: &str) -> Result<()> {
        let table = self.chunk_manager.create_table(name)?;

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

        // prevent duplicates
        if self.catalog
            .columns_by_table
            .get(&table_id)
            .map(|cols| cols.iter().any(|c| c.name == column_name))
            .unwrap_or(false)
        {
            return Ok(());
        }

        let cols = self.catalog.columns_by_table.get(&table_id).unwrap();

        let col = self.chunk_manager.add_column(
            table_id,
            column_name,
            column_type,
            cols.len() as u16
        )?;

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
                ("is_active", ColumnType::Boolean),
            ])
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

        // let test_row = vec![
        //     ("id", Value::Int64(1)),
        //     ("first_name", Value::String("Noubar".into())),
        //     ("last_name", Value::String("Kassabian".into())),
        //     ("email", Value::String("noubar@example.com".into())),
        //     ("password_hash", Value::String("hashed_password_123".into())),
        //     ("is_active", Value::Bool(true)),
        // ];

        // self.append_row("users", test_row)?;

        Ok(())
    }
}

// pub enum EncodedValue {
//     Bytes(Vec<u8>),
//     Null,
// }
//
// #[derive(Debug, Clone)]
// pub enum Value {
//     Int32(i32),
//     Int64(i64),
//     Float64(f64),
//     Bool(bool),
//     String(String),
//     Null,
// }

// impl Value {
//     pub fn matches_column_type(&self, column_type: ColumnType) -> bool {
//         match (self, column_type) {
//             (Value::Int32(_), ColumnType::Integer32) => true,
//             (Value::Int64(_), ColumnType::Integer64) => true,
//             (Value::Float64(_), ColumnType::Float64) => true,
//             (Value::Bool(_), ColumnType::Boolean) => true,
//             (Value::String(_), ColumnType::Utf8) => true,
//             (Value::Null, _) => true,
//             _ => false,
//         }
//     }
//
//     pub fn encode(&self) -> EncodedValue {
//         match self {
//             Value::Int32(v) => EncodedValue::Bytes(v.to_le_bytes().to_vec()),
//             Value::Int64(v) => EncodedValue::Bytes(v.to_le_bytes().to_vec()),
//             Value::Float64(v) => EncodedValue::Bytes(v.to_le_bytes().to_vec()),
//             Value::Bool(v) => EncodedValue::Bytes(vec![*v as u8]),
//             Value::String(s) => {
//                 let mut buf = Vec::with_capacity(4 + s.len());
//                 buf.extend_from_slice(&(s.len() as u32).to_le_bytes());
//                 buf.extend_from_slice(s.as_bytes());
//                 EncodedValue::Bytes(buf)
//             }
//             Value::Null => EncodedValue::Null,
//         }
//     }
// }
//
// pub struct ActiveChunk {
//     // Identity
//     pub table_id: u32,
//     pub column_ordinal: u16,
//
//     // Physical layout
//     pub first_page_id: u32,
//     pub pages: Vec<u32>, // chunk may span multiple pages
//
//     // Runtime state
//     pub value_count: u32,
//
//     // Runtime stats (finalized on seal)
//     pub min: Option<Value>,
//     pub max: Option<Value>,
// }
//
// impl ActiveChunk {
//     pub fn append(&mut self, database: Database, value: &Value) -> Result<()> {
//         let encoded = value.encode();
//
//         let bytes = match encoded {
//             EncodedValue::Bytes(bytes) => bytes,
//             EncodedValue::Null => return Ok(()),
//         };
//
//         let required_space = bytes.len() as u64;
//
//         let page_id = self.pages.last().unwrap().clone() as u64;
//         let page = database.pager.read_page(page_id).unwrap();
//         let page_header = ChunkDataHeader::read_from(&page.buf[PageHeader::SIZE..PageHeader::SIZE + ChunkDataHeader::SIZE]);
//
//         if required_space > page.header
//
//
//     }
// }