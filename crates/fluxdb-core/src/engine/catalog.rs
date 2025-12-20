use std::collections::HashMap;
use crate::metadata::schema::table_column::TableColumn;
use crate::metadata::schema::table_meta::TableMeta;
pub struct Catalog {
    pub tables_by_id: HashMap<u32, TableMeta>,
    pub tables_by_name: HashMap<String, u32>,
    pub columns_by_table: HashMap<u32, Vec<TableColumn>>,
}