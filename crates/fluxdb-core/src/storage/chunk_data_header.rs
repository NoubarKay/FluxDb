
#[repr(C)]
pub struct ChunkDataHeader {
    pub table_id: u32,
    pub column_ordinal: u16,
    pub value_count: u16,
    pub encoding: u8,
    pub flags: u8,
    pub next_page_id: u32,
}

impl ChunkDataHeader{
    pub const SIZE: usize = 4 + 2 + 2 + 1 + 1 + 4;

    pub fn new(table_id: u32, ordinal: u16) -> Self {
        Self {
            table_id,
            column_ordinal: ordinal,
            value_count: 0,
            encoding: 0,
            flags: 0,
            next_page_id: 0,
        }
    }

    pub fn write_to(&self, buf: &mut [u8]) {
        buf[0..4].copy_from_slice(&self.table_id.to_le_bytes());
        buf[4..6].copy_from_slice(&self.column_ordinal.to_le_bytes());
        buf[6..8].copy_from_slice(&self.value_count.to_le_bytes());
        buf[8..9].copy_from_slice(&[self.encoding]);
        buf[9..10].copy_from_slice(&[self.flags]);
        buf[10..14].copy_from_slice(&self.next_page_id.to_le_bytes());
    }

    pub fn read_from(buf: &[u8]) -> Self {
        Self {
            table_id: u32::from_le_bytes(buf[0..4].try_into().unwrap()),
            column_ordinal: u16::from_le_bytes(buf[4..6].try_into().unwrap()),
            value_count: u16::from_le_bytes(buf[6..8].try_into().unwrap()),
            encoding: buf[8],
            flags: buf[9],
            next_page_id: u32::from_le_bytes(buf[10..14].try_into().unwrap()),
        }
    }

}