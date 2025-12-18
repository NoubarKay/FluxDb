use crate::pager::page_type::PageType;

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RecordType {
    CatalogRoot = 0,
    CatalogTable = 1,
    CatalogColumn = 2,
    HeapRow = 10,
    IndexEntry = 20,
}

impl RecordType {
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => RecordType::CatalogRoot,
            1 => RecordType::CatalogTable,
            2 => RecordType::CatalogColumn,
            10 => RecordType::HeapRow,
            20 => RecordType::IndexEntry,
            _ => RecordType::CatalogTable, // or panic, your call
        }
    }
}