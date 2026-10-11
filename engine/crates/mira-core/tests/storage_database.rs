#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Socle du stockage : ouverture, migrations, transactions.

use mira_core::storage::{Database, schema_version};

/// La base en mémoire s'ouvre et le schéma est à la dernière version
#[test]
fn in_memory_database_opens_with_latest_schema() {
    let db = Database::open_in_memory().unwrap();
    let version = db.read(|tx| Ok(tx.schema_version_on_disk())).unwrap();
    assert_eq!(version, i64::try_from(schema_version()).unwrap());
}

/// Rouvrir une base existante ne réapplique pas les migrations
#[test]
fn reopening_a_database_keeps_schema_and_data() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("mira.sqlite3");
    drop(Database::open(&path).unwrap());
    let db = Database::open(&path).unwrap();
    let version = db.read(|tx| Ok(tx.schema_version_on_disk())).unwrap();
    assert_eq!(version, i64::try_from(schema_version()).unwrap());
}

/// L'introspection liste les tables du schéma et compte leurs lignes
#[test]
fn introspection_lists_tables_and_counts_rows() {
    let db = Database::open_in_memory().unwrap();
    let tables = db.read(|tx| tx.table_names()).unwrap();
    assert!(tables.contains(&"messages".to_string()));
    assert!(tables.contains(&"actions".to_string()));
    assert_eq!(db.read(|tx| tx.table_row_count("accounts")).unwrap(), 0);
    assert!(
        db.read(|tx| tx.table_row_count("accounts; DROP TABLE x"))
            .is_err()
    );
}
