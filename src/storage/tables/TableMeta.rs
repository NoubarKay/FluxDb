
pub struct TableMeta {
    pub table_id: u32,
    pub name: String,
}

impl TableMeta {
    pub fn serialize(&self) -> Vec<u8>{
        let mut buf = Vec::new();

        buf.extend(&self.table_id.to_le_bytes());

        let name_bytes = self.name.as_bytes();
        buf.extend(&(name_bytes.len() as u16).to_le_bytes());
        buf.extend(name_bytes);

        buf
    }
    pub fn deserialize(bytes: &[u8]) -> Self {
        assert!(bytes.len() >= 6, "TableMeta record corrupted");

        let table_id =
            u32::from_le_bytes(bytes[0..4].try_into().unwrap());

        let name_len =
            u16::from_le_bytes(bytes[4..6].try_into().unwrap()) as usize;

        let name_start = 6;
        let name_end = name_start + name_len;
        

        let name = String::from_utf8(
            bytes[name_start..name_end].to_vec()
        ).expect("Invalid UTF-8 in table name");

        Self {
            table_id,
            name,
        }
    }
}