use crate::metadata::db_record::DbRecord;
use crate::metadata::record_type::RecordType;

pub struct TableMeta {
    pub table_id: u32,
    pub name: String,
}

impl DbRecord for TableMeta {
    const RECORD_TYPE: RecordType = RecordType::CatalogTable;

    fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(4 + self.name.len());
        buf.extend_from_slice(&self.table_id.to_le_bytes());
        buf.extend_from_slice(self.name.as_bytes());
        buf
    }

    fn deserialize(payload: &[u8]) -> Result<Self, String> {
        let id = u32::from_le_bytes(payload[0..4].try_into().unwrap());
        let name =
            std::str::from_utf8(&payload[4..])
                .map_err(|_| "utf8 error")?
                .to_string();

        Ok(Self { table_id: id, name })
    }
}