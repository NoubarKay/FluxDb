use crate::metadata::record_type::RecordType;

pub trait DbRecord: Sized {
    const RECORD_TYPE: RecordType;

    fn serialize(&self) -> Vec<u8>;
    fn deserialize(payload: &[u8]) -> Result<Self, String>;
}