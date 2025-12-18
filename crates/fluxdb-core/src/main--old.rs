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
    let mut db = Database::open(Path::new("../../test.flxdb"), true)?;

    seed_schema(&mut db)?;

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
    let start = Instant::now();
    for (name, _) in &db.catalog.tables_by_name {
        let table_name: &str = name.as_str();

        if let Some(table_id) = db.catalog.tables_by_name.get(table_name) {
            let table = &db.catalog.tables_by_id[table_id];
            println!("Table '{}' (id={})", table_name,  table.table_id);

            if let Some(cols) = db.catalog.columns_by_table.get(table_id) {
                for col in cols {
                    println!("  - {} (id={})", col.name, col.column_id);
                }
            }
        }
    }

    let functionaltest = start.elapsed();
    println!("Functional lookup took: {:?}", functionaltest);

    // 4️⃣ Schema mutation test
    println!("\n=== Schema Mutation Test ===");

    let start = Instant::now();
    db.create_table("clientsss")?;
    let t3 = start.elapsed();

    println!("CREATE TABLE took: {:?}", t3);

    let table_id = db.catalog.tables_by_name["clientsss"];
    let table = &db.catalog.tables_by_id[&table_id];

    println!(
        "New table loaded in catalog: {} (id={})",
        table.name,
        table.table_id
    );

    // 5️⃣ Reload & compare (disk ↔ memory)
    println!("\n=== Disk Reload Consistency ===");

    let mut db2 = Database::open(Path::new("../../test.flxdb"), false)?;

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

fn seed_schema(db: &mut Database) -> std::io::Result<()> {
    use crate::records::table_column::TableColumn;

    let tables = [
        ("users", vec![
            ("id"),
            ("first_name"),
            ("last_name"),
            ("email"),
            ("password_hash"),
            ("dob"),
            ("is_active"),
            ("created_at"),
            ("updated_at"),
            ("deleted_at"),
        ]),
        ("orders", vec![
            ("id"),
            ("user_id"),
            ("status"),
            ("subtotal"),
            ("tax"),
            ("total"),
            ("currency"),
            ("created_at"),
            ("updated_at"),
            ("deleted_at"),
        ]),
        ("products", vec![
            ("id"),
            ("sku"),
            ("name"),
            ("description"),
            ("price"),
            ("stock"),
            ("category_id"),
            ("is_active"),
            ("created_at"),
            ("updated_at"),
        ]),
        ("categories", vec![
            ("id"),
            ("name"),
            ("slug"),
            ("parent_id"),
            ("sort_order"),
            ("is_active"),
            ("created_at"),
            ("updated_at"),
            ("deleted_at"),
            ("metadata"),
        ]),
        ("payments", vec![
            ("id"),
            ("order_id"),
            ("provider"),
            ("provider_ref"),
            ("amount"),
            ("currency"),
            ("status"),
            ("paid_at"),
            ("created_at"),
            ("updated_at"),
        ]),
        ("addresses", vec![
            ("id"),
            ("user_id"),
            ("line1"),
            ("line2"),
            ("city"),
            ("country"),
            ("postal_code"),
            ("is_default"),
            ("created_at"),
            ("updated_at"),
        ]),
        ("sessions", vec![
            ("id"),
            ("user_id"),
            ("token"),
            ("ip_address"),
            ("user_agent"),
            ("expires_at"),
            ("revoked_at"),
            ("created_at"),
            ("updated_at"),
            ("last_seen_at"),
        ]),
        ("roles", vec![
            ("id"),
            ("name"),
            ("description"),
            ("is_system"),
            ("created_at"),
            ("updated_at"),
            ("deleted_at"),
            ("permissions"),
            ("priority"),
            ("metadata"),
        ]),
        ("user_roles", vec![
            ("id"),
            ("user_id"),
            ("role_id"),
            ("assigned_by"),
            ("assigned_at"),
            ("expires_at"),
            ("is_active"),
            ("created_at"),
            ("updated_at"),
            ("deleted_at"),
        ]),
        ("audit_logs", vec![
            ("id"),
            ("actor_id"),
            ("action"),
            ("entity"),
            ("entity_id"),
            ("payload"),
            ("ip_address"),
            ("created_at"),
            ("request_id"),
            ("severity"),
        ]),
    ];

    for (table_name, columns) in tables {
        if !db.catalog.tables_by_name.contains_key(table_name) {
            db.create_table(table_name)?;
        }

        for (col_name) in columns {
            db.add_column(
                table_name,
                TableColumn {
                    column_id: 0, // assigned internally
                    table_id: 0,  // resolved internally
                    name: col_name.to_string()
                },
            )?;
        }
    }

    Ok(())
}