use crate::storage::page_type::PageType;

#[repr(C)]
pub struct PageHeader{
    pub page_type: PageType,
    pub slot_count: u16,
    pub free_start: u16,
    pub free_end: u16,
    pub page_id: u32,
    pub next_page_id: u32,
    pub reserved: [u8; 9],
}

impl PageHeader{
    pub const SIZE: usize = 24;

    pub fn new(free_start: u16, free_end: u16, page_type: PageType, page_id: u32) -> Self{
        Self{page_type, 
            slot_count: 0, 
            free_start, 
            free_end, 
            page_id, 
            next_page_id: 0, 
            reserved: [0u8; 9]}
    }

    pub fn write_to(&self, buf: &mut [u8]) {
        assert!(buf.len() >= Self::SIZE);

        buf[0] = self.page_type as u8;

        buf[1..3].copy_from_slice(&self.slot_count.to_le_bytes());
        buf[3..5].copy_from_slice(&self.free_start.to_le_bytes());
        buf[5..7].copy_from_slice(&self.free_end.to_le_bytes());
        buf[7..11].copy_from_slice(&self.page_id.to_le_bytes());
        buf[11..15].copy_from_slice(&self.next_page_id.to_le_bytes());
        buf[15..24].copy_from_slice(&self.reserved);
    }

    pub fn read_from(buf: &[u8]) -> Self {
        assert!(buf.len() >= Self::SIZE);

        let page_type = PageType::from_u8(buf[0]);

        let slot_count = u16::from_le_bytes(buf[1..3].try_into().unwrap());
        let free_start = u16::from_le_bytes(buf[3..5].try_into().unwrap());
        let free_end = u16::from_le_bytes(buf[5..7].try_into().unwrap());
        let page_id = u32::from_le_bytes(buf[7..11].try_into().unwrap());
        let next_page_id = u32::from_le_bytes(buf[11..15].try_into().unwrap());

        let mut reserved = [0u8; 9];
        reserved.copy_from_slice(&buf[15..24]);

        Self {
            page_type,
            slot_count,
            free_start,
            free_end,
            page_id,
            next_page_id,
            reserved,
        }
    }
}