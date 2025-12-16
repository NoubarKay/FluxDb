

#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ColumnType{
    Int32 = 1,
    Int64 = 2,
    Float64 = 3,
    Bool = 4,
    Varchar = 5,
}

impl ColumnType{
    
    pub fn from_u8(val: u8) -> Option<ColumnType>{
        match val{
            1 => Some(ColumnType::Int32),
            2 => Some(ColumnType::Int64),
            3 => Some(ColumnType::Float64),
            4 => Some(ColumnType::Bool),
            5 => Some(ColumnType::Varchar),
            _ => None,
        }
    }
    pub fn is_fixed_size(&self) -> bool{
        !matches!(self, ColumnType::Varchar)
    }

    pub fn fixed_size(&self) -> Option<u16>{
        match self{
            ColumnType::Int32 => Some(4),
            ColumnType::Int64 => Some(8),
            ColumnType::Float64 => Some(8),
            ColumnType::Bool => Some(1),
            ColumnType::Varchar => None,
        }
    }
}