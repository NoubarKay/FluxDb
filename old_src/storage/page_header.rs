use crate::storage::slot::Slot;

#[repr(C)]
pub struct PageHeader{
    pub page_type: PageType,
    pub slot_count: u16,
    pub free_start: u16,
    pub free_end: u16,
    pub page_id: u32,
}

impl PageHeader{
    pub const SIZE: usize = 32;

    pub fn write_to(&self, buf: &mut [u8]){
        assert!(buf.len() >= Self::SIZE);

        buf[0..2].copy_from_slice(&(self.page_type as u16).to_le_bytes());
        buf[2..4].copy_from_slice(&(self.slot_count).to_le_bytes());
        buf[4..6].copy_from_slice(&(self.free_start).to_le_bytes());
        buf[6..8].copy_from_slice(&(self.free_end).to_le_bytes());
        buf[8..12].copy_from_slice(&(self.page_id).to_le_bytes());
    }

    pub fn read_from(buf: &[u8]) -> Self {
        assert!(buf.len() >= Self::SIZE);

        let page_type = match u16::from_le_bytes(buf[0..2].try_into().unwrap()) {
            1 => PageType::DataPage,
            2 => PageType::IndexPage,
            3 => PageType::CatalogPage,
            v => panic!("Invalid page type: {}", v),
        };

        Self {
            page_type,
            slot_count: u16::from_le_bytes(buf[2..4].try_into().unwrap()),
            free_start: u16::from_le_bytes(buf[4..6].try_into().unwrap()),
            free_end: u16::from_le_bytes(buf[6..8].try_into().unwrap()),
            page_id: u32::from_le_bytes(buf[8..12].try_into().unwrap()),
        }
    }
}

