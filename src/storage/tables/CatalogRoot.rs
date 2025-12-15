pub struct CatalogRoot {
    pub next_table_id: u32,
    pub next_column_id: u16,
}

impl CatalogRoot {
    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(6);

        buf.extend(&self.next_table_id.to_le_bytes());   // 4 bytes
        buf.extend(&self.next_column_id.to_le_bytes()); // 2 bytes

        buf
    }

    pub fn deserialize(bytes: &[u8]) -> Self {
        // This must be exact — anything else means corruption
        assert!(
            bytes.len() >= 6,
            "CatalogRoot record is corrupted or incomplete"
        );

        let next_table_id =
            u32::from_le_bytes(bytes[0..4].try_into().unwrap());

        let next_column_id =
            u16::from_le_bytes(bytes[4..6].try_into().unwrap());

        Self {
            next_table_id,
            next_column_id,
        }
    }
}