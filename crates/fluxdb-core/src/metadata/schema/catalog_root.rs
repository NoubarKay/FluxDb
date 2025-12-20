use crate::metadata::db_record::DbRecord;
use crate::metadata::record_type::RecordType;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CatalogRoot {
    pub version: u16,
    pub next_table_id: u32,
    pub next_column_id: u32,
    pub catalog_root_page_id: u32,
}

impl DbRecord for CatalogRoot {
    const RECORD_TYPE: RecordType = RecordType::CatalogRoot;

    fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(14);
        buf.extend_from_slice(&self.version.to_le_bytes());
        buf.extend_from_slice(&self.next_table_id.to_le_bytes());
        buf.extend_from_slice(&self.next_column_id.to_le_bytes());
        buf.extend_from_slice(&self.catalog_root_page_id.to_le_bytes());
        buf
    }

    fn deserialize(payload: &[u8]) -> Result<Self, String> {
        let version = u16::from_le_bytes(payload[0..2].try_into().unwrap());
        let next_table_id = u32::from_le_bytes(payload[2..6].try_into().unwrap());
        let next_column_id = u32::from_le_bytes(payload[6..10].try_into().unwrap());
        let catalog_root_page_id = u32::from_le_bytes(payload[10..14].try_into().unwrap());
        Ok(Self { version, next_table_id, next_column_id, catalog_root_page_id })
    }
}