use std::fs::OpenOptions;
use std::path::Path;
use std::time::Instant;
use crate::general::catalog::Catalog;
use crate::general::database::Database;
use crate::records::db_record::DbRecord;

mod general;
mod helpers;
mod pager;
mod records;

fn main() -> std::io::Result<()> {
    let mut db = Database::open(Path::new("test.flxdb"))?;

    println!("=== Catalog Cache Tests ===");
    let c = &db.catalog;

    // 1️⃣ Cached catalog access timing
    let start = Instant::now();
    let catalog1 = &db.catalog;
    let t1 = start.elapsed();

    let start = Instant::now();
    let catalog2 = &db.catalog;
    let t2 = start.elapsed();

    println!("Cached catalog access #1: {:?}", t1);
    println!("Cached catalog access #2: {:?}", t2);

    assert!(
        t1 < std::time::Duration::from_micros(10),
        "t1 cached catalog access is too slow"
    );
    assert!(
        t2 < std::time::Duration::from_micros(1),
        "t2 cached catalog access is too slow"
    );

    println!(
        "Same catalog instance: {}",
        std::ptr::eq(catalog1, catalog2)
    );

    // 2️⃣ Index integrity checks
    println!("\n=== Catalog Index Integrity ===");

    for (name, table_id) in &db.catalog.tables_by_name {
        let table = db.catalog.tables_by_id.get(table_id)
            .expect("table_id missing in tables_by_id");
        assert_eq!(
            &table.name,
            name,
            "table name mismatch between indexes"
        );
    }

    for (table_id, columns) in &db.catalog.columns_by_table {
        assert!(
            db.catalog.tables_by_id.contains_key(table_id),
            "columns exist for unknown table_id={table_id}"
        );

        let mut seen = std::collections::HashSet::new();
        for col in columns {
            assert!(
                seen.insert(col.column_id),
                "duplicate column_id {} in table_id {}",
                col.column_id,
                table_id
            );
        }
    }

    println!("Catalog indexes validated ✔");

    // 3️⃣ Functional lookup test
    println!("\n=== Functional Lookup Test ===");

    if let Some(table_id) = db.catalog.tables_by_name.get("users") {
        let table = &db.catalog.tables_by_id[table_id];
        println!("Table 'users' (id={})", table.table_id);

        if let Some(cols) = db.catalog.columns_by_table.get(table_id) {
            for col in cols {
                println!("  - {} (id={})", col.name, col.column_id);
            }
        }
    }

    // 4️⃣ Schema mutation test
    println!("\n=== Schema Mutation Test ===");

    let start = Instant::now();
    db.create_table("clients")?;
    let t3 = start.elapsed();

    println!("CREATE TABLE took: {:?}", t3);

    let table_id = db.catalog.tables_by_name["clients"];
    let table = &db.catalog.tables_by_id[&table_id];

    println!(
        "New table loaded in catalog: {} (id={})",
        table.name,
        table.table_id
    );

    // 5️⃣ Reload & compare (disk ↔ memory)
    println!("\n=== Disk Reload Consistency ===");

    let mut db2 = Database::open(Path::new("test.flxdb"))?;

    assert_eq!(
        db.catalog.tables_by_id.len(),
        db2.catalog.tables_by_id.len(),
        "table count mismatch after reload"
    );

    assert_eq!(
        db.catalog.columns_by_table.len(),
        db2.catalog.columns_by_table.len(),
        "column count mismatch after reload"
    );

    println!("Disk reload consistency verified ✔");

    // 6️⃣ Performance regression test
    println!("\n=== Performance Regression Test ===");

    let iterations = 100_000;
    let start = Instant::now();

    for _ in 0..iterations {
        let table_id = db.catalog.tables_by_name["users"];
        let _cols = &db.catalog.columns_by_table[&table_id];
    }

    let avg = start.elapsed() / iterations;
    println!("Avg indexed lookup: {:?}", avg);

    assert!(
        avg < std::time::Duration::from_micros(1),
        "indexed lookup too slow"
    );

    println!("\nAll catalog tests passed ✔");

    Ok(())
}