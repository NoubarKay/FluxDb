use std::collections::HashMap;
use std::io::Error;
use crate::engine::catalog::Catalog;
use crate::metadata::chunks::active_chunk::ActiveChunk;
use crate::metadata::schema::column_type::ColumnType;
use crate::metadata::schema::table_column::TableColumn;
use crate::metadata::schema::table_meta::TableMeta;
use crate::storage::pager::Pager;

pub struct ChunkManager {
    pub pager: Pager,
    pub active_chunks: HashMap<(u32, u16), ActiveChunk>,
}

impl ChunkManager {
    pub fn new(pager: Pager) -> Self {
        Self { pager, active_chunks: HashMap::new() }
    }

    pub fn load_catalog(&mut self) -> Result<Catalog, Error> {
        self.pager.load_catalog()
    }

    pub fn init_catalog_root(&mut self) -> Result<(), Error> {
        self.pager.init_catalog_root()
    }

    pub fn create_table(&mut self, p0: &str) -> Result<TableMeta, Error> {
        self.pager.create_table(p0)
    }

    pub fn add_column(&mut self, table_id: u32, col_name: &str, col_type: ColumnType, ordinal: u16) -> Result<TableColumn, Error> {
        self.pager.add_column(table_id, col_name, col_type, ordinal)
    }
}