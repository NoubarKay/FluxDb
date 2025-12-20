#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ColumnType {
    Integer32,
    Integer64,
    Float32,
    Float64,
    Utf8,
    Timestamp,
    Boolean
}

impl ColumnType {
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => ColumnType::Integer32,
            1 => ColumnType::Integer64,
            2 => ColumnType::Float32,
            3 => ColumnType::Float64,
            4 => ColumnType::Utf8,
            5 => ColumnType::Timestamp,
            6 => ColumnType::Boolean,
            _ => panic!(), // or panic, your call
        }
    }
}