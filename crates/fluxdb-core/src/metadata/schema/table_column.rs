use crate::metadata::db_record::DbRecord;
use crate::metadata::record_type::RecordType;
use crate::metadata::schema::column_type::ColumnType;

pub struct TableColumn {
    pub table_id: u32,
    pub column_id: u32,
    pub column_type: ColumnType,
    pub name: String,
}

impl DbRecord for TableColumn {
    const RECORD_TYPE: RecordType = RecordType::CatalogColumn;

    fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(12);
        buf.extend_from_slice(&self.table_id.to_le_bytes());
        buf.extend_from_slice(&self.column_id.to_le_bytes());
        buf.extend_from_slice(&[self.column_type as u8]);
        buf.extend_from_slice(self.name.as_bytes());
        buf
    }

    fn deserialize(payload: &[u8]) -> Result<Self, String> {
        let table_id = u32::from_le_bytes(payload[0..4].try_into().unwrap());
        let column_id = u32::from_le_bytes(payload[4..8].try_into().unwrap());
        let column_type_raw = u8::from_le_bytes(payload[8..9].try_into().unwrap());
        let string = std::str::from_utf8(&payload[9..])
            .map_err(|_| "utf8 error")?
            .to_string();

        let column_type = ColumnType::from_u8(column_type_raw);
        
        Ok(Self { table_id, column_id, column_type, name: string })
    }
}