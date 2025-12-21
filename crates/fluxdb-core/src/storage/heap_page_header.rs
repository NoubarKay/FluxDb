use crate::storage::page_header::PageHeader;

pub struct HeapPageHeader{
    pub slot_count: u16,
    pub free_start: u16,
    pub free_end: u16,
}

impl HeapPageHeader {
    pub const SIZE: usize = 2 + 2 + 2; // 6 bytes

    pub fn new(page_size: usize) -> Self {
        Self {
            slot_count: 0,
            free_start: (PageHeader::SIZE + Self::SIZE) as u16,
            free_end: page_size as u16,
        }
    }

    /// MAKE SURE TO GIVE THE BUFFER WITHOUT THE HEADER
    pub fn write_to(&self, buf: &mut [u8]) {
        buf[0..2].copy_from_slice(&self.slot_count.to_le_bytes());
        buf[2..4].copy_from_slice(&self.free_start.to_le_bytes());
        buf[4..6].copy_from_slice(&self.free_end.to_le_bytes());
    }

    pub fn read_from(buf: &[u8]) -> Self {
        Self {
            slot_count: u16::from_le_bytes(buf[0..2].try_into().unwrap()),
            free_start: u16::from_le_bytes(buf[2..4].try_into().unwrap()),
            free_end: u16::from_le_bytes(buf[4..6].try_into().unwrap()),
        }
    }
}