use std::fs::write;
use std::io::{Read, Write};
use crate::storage::tables::ColumnType::ColumnType;

pub struct ColumnMeta {
    pub table_id: u32,
    pub column_id: u16,
    pub name_length: u8,
    pub name: Vec<u8>,
    pub data_type: ColumnType,
    pub reserved: [u8; 20],
}

impl ColumnMeta {
    pub fn write_to<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&self.table_id.to_le_bytes())?;
        writer.write_all(&self.column_id.to_le_bytes())?;

        writer.write_all(&[self.name_length])?;
        writer.write_all(&self.name)?;

        writer.write_all(&[self.data_type as u8])?;
        writer.write_all(&self.reserved)?;

        Ok(())
    }

    pub fn serialize(&self) -> std::io::Result<Vec<u8>> {
        let mut buf = Vec::new();
        self.write_to(&mut buf)?;
        Ok(buf)
    }

    pub fn read_from<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut buf4 = [0u8; 4];
        let mut buf2 = [0u8; 2];
        let mut buf1 = [0u8; 1];

        // table_id
        reader.read_exact(&mut buf4)?;
        let table_id = u32::from_le_bytes(buf4);

        // column_id
        reader.read_exact(&mut buf2)?;
        let column_id = u16::from_le_bytes(buf2);

        // name_length
        reader.read_exact(&mut buf1)?;
        let name_length = buf1[0];

        // name
        let mut name = vec![0u8; name_length as usize];
        reader.read_exact(&mut name)?;

        // data_type
        reader.read_exact(&mut buf1)?;
        let data_type = ColumnType::from_u8(buf1[0]).unwrap();

        // reserved
        let mut reserved = [0u8; 20];
        reader.read_exact(&mut reserved)?;

        Ok(Self {
            table_id,
            column_id,
            name,
            data_type,
            name_length,
            reserved,
        })
    }
}