use crate::records::db_record::DbRecord;
use crate::records::record_type::RecordType;

pub struct TableColumn {
    pub table_id: u32,
    pub column_id: u32,
    pub name: String,
}

impl DbRecord for TableColumn {
    const RECORD_TYPE: RecordType = RecordType::CatalogColumn;

    fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(12);
        buf.extend_from_slice(&self.table_id.to_le_bytes());
        buf.extend_from_slice(&self.column_id.to_le_bytes());
        buf.extend_from_slice(self.name.as_bytes());
        buf
    }

    fn deserialize(payload: &[u8]) -> Result<Self, String> {
        let table_id = u32::from_le_bytes(payload[0..4].try_into().unwrap());
        let column_id = u32::from_le_bytes(payload[4..8].try_into().unwrap());
        let string = std::str::from_utf8(&payload[8..])
            .map_err(|_| "utf8 error")?
            .to_string();
        
        Ok(Self { table_id, column_id, name: string })
    }
}