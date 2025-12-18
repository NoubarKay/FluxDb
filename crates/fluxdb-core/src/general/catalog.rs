use std::collections::HashMap;
use crate::records::table_column::TableColumn;
use crate::records::table_meta::TableMeta;

pub struct Catalog {
    pub tables_by_id: HashMap<u32, TableMeta>,
    pub tables_by_name: HashMap<String, u32>,
    pub columns_by_table: HashMap<u32, Vec<TableColumn>>,
}