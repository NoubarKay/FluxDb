mod storage;

use std::fs::File;
use std::path::Path;
use crate::storage::file::{create_db_file, open_db_file};
use crate::storage::header::FluxDbFileHeader;
use crate::storage::page::Page;
use crate::storage::page_header::PageType;
use crate::storage::pager::FluxPager;
use crate::storage::tables::CatalogRoot::CatalogRoot;
use crate::storage::tables::TableMeta::TableMeta;
use crate::storage::error::FluxError;
use crate::storage::tables::ColumnMeta::ColumnMeta;
use crate::storage::tables::ColumnType::ColumnType;

fn main() -> FluxError::Result<()> {
    let path = Path::new("test.flxdb");

    let mut pager = init_db_file(path)?;
    
    let page = pager.read_page(0).unwrap();

    print_table_meta_from_slot(&page, 1).unwrap();
    print_column_meta_from_slot(&page, 2).unwrap();
    print_column_meta_from_slot(&page, 3).unwrap();
    print_column_meta_from_slot(&page, 4).unwrap();
    
    Ok(())
}

pub fn init_db_file(path: &Path) -> FluxError::Result<FluxPager> {
    let mut file = create_db_file(path)?;
    let header = FluxDbFileHeader::new(4096);
    header.write_to(&mut file)?;

    let mut header = FluxDbFileHeader::read_from(&mut file)?;

    let mut pager = FluxPager::new(file, header);

    pager.init_catalog_root().expect("TODO: panic message");

    let columns = vec![
        ColumnMeta {
            table_id: 0, // will be assigned inside create_table
            column_id: 0, // will be assigned inside create_table
            name_length: 2,
            name: b"id".to_vec(),
            data_type: ColumnType::Int64,
            reserved: [0; 20],
        },
        ColumnMeta {
            table_id: 0,
            column_id: 0,
            name_length: 5,
            name: b"email".to_vec(),
            data_type: ColumnType::Varchar,
            reserved: [0; 20],
        },
        ColumnMeta {
            table_id: 0,
            column_id: 0,
            name_length: 10,
            name: b"first_name".to_vec(),
            data_type: ColumnType::Varchar,
            reserved: [0; 20],
        },
        ColumnMeta {
            table_id: 0,
            column_id: 0,
            name_length: 9,
            name: b"last_name".to_vec(),
            data_type: ColumnType::Varchar,
            reserved: [0; 20],
        },
        ColumnMeta {
            table_id: 0,
            column_id: 0,
            name_length: 10,
            name: b"created_at".to_vec(),
            data_type: ColumnType::Int64, // unix timestamp for now
            reserved: [0; 20],
        },
    ];

    pager.create_table("customers", &columns).unwrap();
    
    Ok(pager)
}

fn print_table_meta_from_slot(page: &Page, slot_id: u16) -> FluxError::Result<()> {
    let slot = page
        .read_slot(slot_id)
        .ok_or_else(|| FluxError::FluxError::NotFound("slot not found"))?;

    let start = slot.offset as usize;
    let end = start + slot.length as usize;
    let record_bytes = &page.buf[start..end];

    let table_meta = TableMeta::deserialize(record_bytes);

    println!("table id: {}", table_meta.table_id);
    println!("table name: {}", table_meta.name);
    Ok(())
}

fn print_column_meta_from_slot(page: &Page, slot_id: u16) -> FluxError::Result<()> {
    let slot = page
        .read_slot(slot_id)
        .ok_or_else(|| FluxError::FluxError::NotFound("slot not found"))?;

    let start = slot.offset as usize;
    let end = start + slot.length as usize;
    let record_bytes = &page.buf[start..end];

    let mut reader = record_bytes;
    let column_meta = ColumnMeta::read_from(&mut reader)?;

    println!("table id   : {}", column_meta.table_id);
    println!("column id  : {}", column_meta.column_id);
    println!(
        "column name: {}",
        String::from_utf8_lossy(&column_meta.name)
    );
    println!("data type  : {:?}", column_meta.data_type);

    Ok(())
}