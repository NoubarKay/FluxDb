#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PageType {
    DataPage    = 1,
    IndexPage   = 2,
    CatalogPage = 3,
}

impl PageType {
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => PageType::DataPage,
            2 => PageType::IndexPage,
            3 => PageType::CatalogPage,
            _ => PageType::DataPage, // or panic, your call
        }
    }
}