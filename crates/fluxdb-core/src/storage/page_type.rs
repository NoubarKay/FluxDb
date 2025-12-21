#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PageType {
    DataPage    = 1,
    HeapPage = 2,
    IndexPage   = 3,
    CatalogPage = 4,
}

impl PageType {
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => PageType::DataPage,
            2 => PageType::HeapPage,
            3 => PageType::IndexPage,
            4 => PageType::CatalogPage,
            _ => PageType::DataPage, // or panic, your call
        }
    }
}