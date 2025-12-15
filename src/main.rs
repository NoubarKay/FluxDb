mod storage;

use std::fs::File;
use std::path::Path;
use crate::storage::file::{create_db_file, open_db_file};
use crate::storage::header::FluxDbFileHeader;
use crate::storage::page_header::PageType;
use crate::storage::pager::FluxPager;
use crate::storage::tables::CatalogRoot::CatalogRoot;
use crate::storage::tables::TableMeta::TableMeta;

fn main() -> std::io::Result<()> {
    let path = Path::new("test.mydb");
    //
    // let mut file = create_db_file(path)?;
    // let header = FluxDbFileHeader::new(4096);
    // header.write_to(&mut file)?;
    //
    // drop(file);

    let mut file = open_db_file(path)?;
    let mut header = FluxDbFileHeader::read_from(&mut file)?;

    let mut pager = FluxPager::new(file, header);

    // pager.init_catalog_root().expect("TODO: panic message");
    //
    // pager.create_table("customers").unwrap();
    // pager.create_table("orders").unwrap();
    // pager.create_table("order_items").unwrap();

    let bytes = pager.load_catalog_root()?.serialize();
    let root = CatalogRoot::deserialize(&*bytes);

    println!("{}", root.next_table_id);

    let page = pager.read_page(0).unwrap();

    // 1. Read slot
    let slot = page.read_slot(1).expect("slot not found");

    // 2. Read record bytes
    let start = (slot.offset) as usize;
    let end = start + slot.length as usize;
    let bytes = &page.buf[start..end];


    // 3. Deserialize TableMeta
    let table_meta = TableMeta::deserialize(&bytes); // skip record tag

    println!("table id: {}", table_meta.table_id);
    println!("table name: {}", table_meta.name);

    let slot = page.read_slot(2).expect("slot not found");

    // 2. Read record bytes
    let start = (slot.offset) as usize;
    let end = start + slot.length as usize;
    let bytes = &page.buf[start..end];

    // 3. Deserialize TableMeta
    let table_meta = TableMeta::deserialize(&bytes); // skip record tag

    println!("table id: {}", table_meta.table_id);
    println!("table name: {}", table_meta.name);

    let slot = page.read_slot(3).expect("slot not found");

    // 2. Read record bytes
    let start = (slot.offset) as usize;
    let end = start + slot.length as usize;
    let bytes = &page.buf[start..end];

    // 3. Deserialize TableMeta
    let table_meta = TableMeta::deserialize(&bytes); // skip record tag

    println!("table id: {}", table_meta.table_id);
    println!("table name: {}", table_meta.name);

    Ok(())
}

