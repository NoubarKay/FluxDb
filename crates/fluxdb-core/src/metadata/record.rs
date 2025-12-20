use crate::metadata::record_type::RecordType;

pub struct Record<'a>{
    pub record_type: RecordType,
    pub payload: &'a [u8]
}
pub const RECORD_HEADER_SIZE: usize = 3;

impl<'a> Record<'a>{
    pub fn encode(record_type: RecordType, payload: &[u8]) -> Vec<u8> {
        let mut buf = Vec::with_capacity(1 + payload.len());
        buf.push(record_type as u8);
        buf.extend_from_slice(payload);
        buf
    }

    pub fn decode(buf: &[u8]) -> Option<(RecordType, &[u8])> {
        if buf.len() < 1 { return None; }

        let record_type = RecordType::from_u8(buf[0]);
        Some((record_type, &buf[1..]))
    }
}