use crate::storage::page_type::PageType;

#[repr(C)]
pub struct PageHeader{
    pub page_type: PageType,
    pub page_id: u32,
    pub next_page_id: u32,
    pub reserved: [u8; 15],
}

impl PageHeader{
    pub const SIZE: usize = 1 + 4 + 4 + 15;

    pub fn new(page_type: PageType, page_id: u32) -> Self {
        Self {
            page_type,
            page_id,
            next_page_id: 0,
            reserved: [0u8; 15],
        }
    }

    pub fn write_to(&self, buf: &mut [u8]) {
        assert!(buf.len() >= Self::SIZE);

        buf[0] = self.page_type as u8;
        buf[1..5].copy_from_slice(&self.page_id.to_le_bytes());
        buf[5..9].copy_from_slice(&self.next_page_id.to_le_bytes());
        buf[9..24].copy_from_slice(&self.reserved);
    }

    pub fn read_from(buf: &[u8]) -> Self {
        assert!(buf.len() >= Self::SIZE);

        let page_type = PageType::from_u8(buf[0]);
        let page_id = u32::from_le_bytes(buf[1..5].try_into().unwrap());
        let next_page_id = u32::from_le_bytes(buf[5..9].try_into().unwrap());

        let mut reserved = [0u8; 15];
        reserved.copy_from_slice(&buf[9..24]);

        Self {
            page_type,
            page_id,
            next_page_id,
            reserved,
        }
    }
}