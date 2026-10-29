#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Rétention du journal des envois d'IA : nombre de lignes, ancienneté, lignes en cours épargnées.

mod support_ai_journal;

use mira_core::storage::Database;
use mira_vocabulary::{OperationId, Timestamp};
use support_ai_journal::{finish, ids, insert_all};

/// Ne restent que les `keep` lignes les plus récentes
#[test]
fn trim_keeps_only_the_latest_n() {
    let db = Database::open_in_memory().unwrap();
    insert_all(
        &db,
        &[
            ("j-1", "a", 1),
            ("j-2", "b", 2),
            ("j-3", "c", 3),
            ("j-4", "d", 4),
        ],
    );
    close_all(&db, &["a", "b", "c", "d"]);
    let removed = db
        .write(|tx| tx.trim_ai_journal(2, Timestamp::from_millis(0)))
        .unwrap();
    assert_eq!(removed, 2);
    let rows = db.read(|tx| tx.list_ai_journal(10, None)).unwrap();
    assert_eq!(ids(&rows), ["j-4", "j-3"]);
}

/// Les lignes antérieures à la date limite partent, même si le nombre est respecté
#[test]
fn trim_drops_rows_older_than_the_date() {
    let db = Database::open_in_memory().unwrap();
    insert_all(
        &db,
        &[("j-1", "a", 100), ("j-2", "b", 200), ("j-3", "c", 300)],
    );
    close_all(&db, &["a", "b", "c"]);
    db.write(|tx| tx.trim_ai_journal(100, Timestamp::from_millis(200)))
        .unwrap();
    let rows = db.read(|tx| tx.list_ai_journal(10, None)).unwrap();
    assert_eq!(ids(&rows), ["j-3", "j-2"]);
}

/// Une ligne `running` n'est jamais supprimée, ni par le nombre ni par la date
#[test]
fn trim_keeps_running_rows() {
    let db = Database::open_in_memory().unwrap();
    insert_all(&db, &[("j-1", "a", 1), ("j-2", "b", 2), ("j-3", "c", 3)]);
    close_all(&db, &["b", "c"]);
    db.write(|tx| tx.trim_ai_journal(1, Timestamp::from_millis(10)))
        .unwrap();
    let rows = db.read(|tx| tx.list_ai_journal(10, None)).unwrap();
    assert_eq!(ids(&rows), ["j-1"]);
    assert_eq!(rows[0].outcome, "running");
}

/// Complète ces opérations avec une issue normale
fn close_all(db: &Database, operations: &[&str]) {
    for name in operations {
        let operation = OperationId::from_raw(*name);
        db.write(|tx| tx.finish_ai_journal(&operation, &finish("end_turn", 0)))
            .unwrap();
    }
}
