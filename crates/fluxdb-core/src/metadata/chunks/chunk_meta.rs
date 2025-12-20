use crate::metadata::db_record::DbRecord;
use crate::metadata::record_type::RecordType;
use crate::metadata::schema::column_type::ColumnType;

pub struct ChunkMeta {
    pub table_id: u32,
    pub column_id: u32,
    pub chunk_id: u32,
    pub row_start: u64,
    pub row_end: u64,
    pub column_type: ColumnType,
    pub first_page_id: u64,
    pub page_count: u64,
}

impl DbRecord for ChunkMeta {
    const RECORD_TYPE: RecordType = RecordType::ChunkMeta;

    fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(4 + 4 + 4 + 8 + 8 + 1 + 8 + 8);

        buf.extend_from_slice(&self.table_id.to_le_bytes());
        buf.extend_from_slice(&self.column_id.to_le_bytes());
        buf.extend_from_slice(&self.chunk_id.to_le_bytes());
        buf.extend_from_slice(&self.row_start.to_le_bytes());
        buf.extend_from_slice(&self.row_end.to_le_bytes());
        buf.push(self.column_type as u8);
        buf.extend_from_slice(&self.first_page_id.to_le_bytes());
        buf.extend_from_slice(&self.page_count.to_le_bytes());

        buf
    }

    fn deserialize(payload: &[u8]) -> Result<Self, String> {
        let mut offset = 0;

        let read_u32 = |buf: &[u8], off: &mut usize| {
            let v = u32::from_le_bytes(buf[*off..*off + 4].try_into().unwrap());
            *off += 4;
            v
        };

        let read_u64 = |buf: &[u8], off: &mut usize| {
            let v = u64::from_le_bytes(buf[*off..*off + 8].try_into().unwrap());
            *off += 8;
            v
        };

        let table_id = read_u32(payload, &mut offset);
        let column_id = read_u32(payload, &mut offset);
        let chunk_id = read_u32(payload, &mut offset);
        let row_start = read_u64(payload, &mut offset);
        let row_end = read_u64(payload, &mut offset);

        let column_type = ColumnType::from_u8(payload[offset]);
        offset += 1;

        let first_page_id = read_u64(payload, &mut offset);
        let page_count = read_u64(payload, &mut offset);

        Ok(Self {
            table_id,
            column_id,
            chunk_id,
            row_start,
            row_end,
            column_type,
            first_page_id,
            page_count,
        })
    }
}
