use std::io;
use crate::storage::page_header::PageHeader;
use crate::storage::slot::Slot;

pub struct Page{
    pub buf: Vec<u8>
}

impl Page{
    pub fn new(buf: Vec<u8>) -> Self{
        Self{buf}
    }
    pub fn insert_record(&mut self, record: &[u8]) -> Result<u16, &'static str>
    {
        let mut header = PageHeader::read_from(&self.buf);

        let available_space = header.free_end - header.free_start;
        let required_space = record.len() as u16 + Slot::SIZE as u16;

        if available_space < required_space {
            return Err("Not enough space on page");
        }

        // 1. Write record
        let record_offset = header.free_start;
        let start = record_offset as usize;
        let end = start + record.len();
        self.buf[start..end].copy_from_slice(record);

        // 2. Write slot
        let slot_offset = header.free_end - Slot::SIZE as u16;
        let slot_start = slot_offset as usize;
        let slot_end = slot_start + Slot::SIZE;

        Slot {
            offset: record_offset,
            length: record.len() as u16,
        }
        .write_to(&mut self.buf[slot_start..slot_end]);

        // 3. Update header
        header.free_start += record.len() as u16;
        header.free_end -= Slot::SIZE as u16;
        header.slot_count += 1;

        header.write_to(&mut self.buf);

        Ok(header.slot_count - 1)
    }


    pub fn read_record(&self, slot_id: u16) -> Option<&[u8]> {
        let slot = self.read_slot(slot_id)?;
        let start = slot.offset as usize;
        let end = start + slot.length as usize;
        Some(&self.buf[start..end])
    }

    pub fn read_slot(&self, slot_id: u16) -> Option<Slot> {
        let header = PageHeader::read_from(&self.buf);

        if slot_id >= header.slot_count {
            return None;
        }

        let page_size = self.buf.len();
        let slot_start =
            page_size - ((slot_id as usize + 1) * Slot::SIZE);

        let slot_end = slot_start + Slot::SIZE;

        Some(Slot::read_from(&self.buf[slot_start..slot_end]))
    }
}