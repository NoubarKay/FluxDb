

pub struct Slot {
    pub offset: u16,
    pub length: u16,
}

impl Slot {
    pub const SIZE: usize = 4;
    pub fn new(offset: u16, length: u16) -> Self {
        Self { offset, length }
    }

    pub fn write_to(&self, buf: &mut [u8]) {
        buf[0..2].copy_from_slice(&(self.offset).to_le_bytes());
        buf[2..4].copy_from_slice(&(self.length).to_le_bytes());
    }
    
    pub fn read_from(buf: &[u8]) -> Self {
        Self { offset: u16::from_le_bytes(buf[0..2].try_into().unwrap()), 
            length: u16::from_le_bytes(buf[2..4].try_into().unwrap()) }
    }
}