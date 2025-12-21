use std::io::Error;
use crate::metadata::db_record::DbRecord;
use crate::metadata::record::Record;
use crate::storage::chunk_data_header::ChunkDataHeader;
use crate::storage::heap_page_header::HeapPageHeader;
use crate::storage::page_header::PageHeader;
use crate::storage::page_type::PageType;
use crate::storage::slot::Slot;

pub struct Page{
    pub header: PageHeader,
    pub buf: Vec<u8>
}

impl Page{
    const HEADER_SIZE: usize = PageHeader::SIZE;
    pub fn new(page_size: usize, page_type: PageType, page_id: u32) -> Self {
        let header = PageHeader::new(
            page_type,
            page_id,
        );

        let mut buf = vec![0u8; page_size];
        header.write_to(&mut buf[..PageHeader::SIZE]);

        match page_type {
            PageType::HeapPage | PageType::CatalogPage => {
                let mut layout = HeapPageHeader::new(page_size);
                layout.write_to(&mut buf[PageHeader::SIZE..]);
            },
            _ => panic!("Unknown page type")
        };

        Self { header, buf }
    }

    pub fn new_chunk_data(page_size: usize, page_id: u32, table_id: u32, ordinal: u16) -> Self {
        let header = PageHeader::new(
            PageType::DataPage,
            page_id,
        );

        let mut buf = vec![0u8; page_size];
        header.write_to(&mut buf[..PageHeader::SIZE]);

        let mut layout = ChunkDataHeader::new(table_id, ordinal);
        layout.write_to(&mut buf[PageHeader::SIZE..]);

        Self { header, buf }
    }

    pub fn insert_typed_record<T: DbRecord>(&mut self, value: &T) -> Result<(), Error>{
        let bytes = Record::encode(T::RECORD_TYPE, &value.serialize());
        self.insert_record(&bytes)
    }

    pub fn insert_record(&mut self, record: &[u8]) -> Result<(), Error> {

        match self.header.page_type {
            PageType::HeapPage | PageType::CatalogPage => {
                self.insert_heap_record(record)?;
            }
            
            _ => panic!("Unknown page type")
        }

        Ok(())
    }

    pub fn read_record(&self, slot_id: u16) -> Option<&[u8]> {

        let page_size = self.buf.len();

        let slot_pos =
            page_size - ((slot_id as usize + 1) * Slot::SIZE);

        let slot = Slot::read_from(
            &self.buf[slot_pos..slot_pos + Slot::SIZE]
        );

        let start = slot.offset as usize;
        let end = start + slot.length as usize;


        Some(&self.buf[start..end])
    }

    pub fn from_buffer(buf: Vec<u8>) -> Self {
        let header = PageHeader::read_from(&buf[..PageHeader::SIZE]);
        Self { header, buf }
    }

    pub fn read_slot(&self, slot_id: u16) -> Option<Slot> {
        let layout = HeapPageHeader::read_from(&self.buf[PageHeader::SIZE..PageHeader::SIZE + HeapPageHeader::SIZE]);
        if slot_id >= layout.slot_count {
            return None;
        }

        let page_size = self.buf.len();
        let slot_pos =
            page_size - ((slot_id as usize + 1) * Slot::SIZE);

        Some(Slot::read_from(
            &self.buf[slot_pos..slot_pos + Slot::SIZE],
        ))
    }

    pub fn iter_slots(&self) -> impl Iterator<Item = (u16, Slot)> + '_ {
        let layout = HeapPageHeader::read_from(&self.buf[PageHeader::SIZE..PageHeader::SIZE + HeapPageHeader::SIZE]);

        (0..layout.slot_count)
            .map(|id| (id, self.read_slot(id).unwrap()))
    }

    fn insert_heap_record(&mut self, record: &[u8]) -> Result<(), Error>{
        let record_len = record.len() as u16;
        let required_space = record_len + Slot::SIZE as u16;
        let mut layout = HeapPageHeader::read_from(&self.buf[PageHeader::SIZE..PageHeader::SIZE + HeapPageHeader::SIZE]);

        let free_space = layout.free_end - layout.free_start;
        if required_space > free_space {
            return Err(Error::new(
                std::io::ErrorKind::Other,
                "Not enough space on page",
            ));
        }

        // 1️⃣ Write record
        let record_offset =layout.free_start as usize;
        self.buf[record_offset..record_offset + record.len()]
            .copy_from_slice(record);

        // 2️⃣ Create slot pointing to record
        let slot = Slot {
            offset: layout.free_start,
            length: record_len,
        };

        let slot_offset = (layout.free_end as usize) - Slot::SIZE;
        slot.write_to(&mut self.buf[slot_offset..slot_offset + Slot::SIZE]);

        // 4️⃣ Update header
        layout.free_start += record_len;
        layout.free_end -= Slot::SIZE as u16;
        layout.slot_count += 1;

        // 5️⃣ Persist header
        layout.write_to(&mut self.buf[PageHeader::SIZE..]);
        Ok(())
    }
}