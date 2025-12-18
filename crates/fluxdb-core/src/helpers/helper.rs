use std::io::Read;

pub fn current_unix_time() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub fn read_u8<R: Read>(r: &mut R) -> u8 {
    let mut b = [0u8; 1];
    r.read_exact(&mut b).unwrap();
    b[0]
}

pub fn read_u16<R: Read>(r: &mut R) -> u16 {
    let mut b = [0u8; 2];
    r.read_exact(&mut b).unwrap();
    u16::from_le_bytes(b)
}

pub fn read_u32<R: Read>(r: &mut R) -> u32 {
    let mut b = [0u8; 4];
    r.read_exact(&mut b).unwrap();
    u32::from_le_bytes(b)
}

pub fn read_u64<R: Read>(r: &mut R) -> u64 {
    let mut b = [0u8; 8];
    r.read_exact(&mut b).unwrap();
    u64::from_le_bytes(b)
}
