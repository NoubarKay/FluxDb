pub struct ColumnMeta {
    table_id: u32,
    column_id: u16,
    name: String,
    // data_type: DataType,
}

impl ColumnMeta {
    fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        buf.extend(&self.table_id.to_le_bytes());

        let name_bytes = self.name.as_bytes();
        buf.extend(&(name_bytes.len() as u16).to_le_bytes());
        buf.extend(name_bytes);

        buf
    }
}