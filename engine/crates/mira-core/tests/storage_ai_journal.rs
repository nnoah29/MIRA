#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Journal des envois d'IA : ouverture, lecture récentes d'abord, complétion, curseur, vidage, reprise.

mod support_ai_journal;

use mira_core::storage::Database;
use mira_vocabulary::{ContentKind, OperationId};
use support_ai_journal::{entry, finish, ids, insert_all};

/// Les lignes sortent de la plus récente à la plus ancienne, à date égale la dernière écrite d'abord
#[test]
fn insert_then_list_newest_first() {
    let db = Database::open_in_memory().unwrap();
    insert_all(
        &db,
        &[
            ("j-1", "op-1", 10),
            ("j-2", "op-2", 30),
            ("j-3", "op-3", 30),
        ],
    );
    let listed = db.read(|tx| tx.list_ai_journal(10, None)).unwrap();
    assert_eq!(ids(&listed), ["j-3", "j-2", "j-1"]);
    assert_eq!(listed[2], entry("j-1", "op-1", 10));
    assert_eq!(listed[2].kinds, [ContentKind::MailMessage]);
}

/// Compléter une ligne fixe l'issue, les jetons et le nombre de valeurs masquées
#[test]
fn finish_completes_the_row() {
    let db = Database::open_in_memory().unwrap();
    insert_all(&db, &[("j-1", "op-1", 10)]);
    let operation = OperationId::from_raw("op-1");
    db.write(|tx| tx.finish_ai_journal(&operation, &finish("end_turn", 3)))
        .unwrap();
    let row = db.read(|tx| tx.list_ai_journal(1, None)).unwrap().remove(0);
    assert_eq!(row.outcome, "end_turn");
    assert_eq!(
        (row.masked, row.input_tokens, row.output_tokens),
        (3, Some(12), Some(5))
    );
    assert_eq!(row.chars, entry("j-1", "op-1", 10).chars);
}

/// Le curseur reprend après la ligne désignée, sans recouvrement ni trou
#[test]
fn list_cursor_pages_without_overlap() {
    let db = Database::open_in_memory().unwrap();
    insert_all(
        &db,
        &[
            ("j-1", "a", 1),
            ("j-2", "b", 2),
            ("j-3", "c", 3),
            ("j-4", "d", 4),
            ("j-5", "e", 5),
        ],
    );
    let first = db.read(|tx| tx.list_ai_journal(2, None)).unwrap();
    let second = db.read(|tx| tx.list_ai_journal(2, Some("j-4"))).unwrap();
    let third = db.read(|tx| tx.list_ai_journal(2, Some("j-2"))).unwrap();
    assert_eq!(ids(&first), ["j-5", "j-4"]);
    assert_eq!(ids(&second), ["j-3", "j-2"]);
    assert_eq!(ids(&third), ["j-1"]);
}

/// Vider le journal supprime toutes les lignes
#[test]
fn clear_empties_the_table() {
    let db = Database::open_in_memory().unwrap();
    insert_all(&db, &[("j-1", "op-1", 10), ("j-2", "op-2", 20)]);
    db.write(|tx| tx.clear_ai_journal()).unwrap();
    assert!(
        db.read(|tx| tx.list_ai_journal(10, None))
            .unwrap()
            .is_empty()
    );
}

/// Les lignes restées `running` passent à `interrupted`, les autres ne bougent pas
#[test]
fn interrupt_marks_running_rows() {
    let db = Database::open_in_memory().unwrap();
    insert_all(&db, &[("j-1", "op-1", 10), ("j-2", "op-2", 20)]);
    let operation = OperationId::from_raw("op-1");
    db.write(|tx| tx.finish_ai_journal(&operation, &finish("end_turn", 0)))
        .unwrap();
    assert_eq!(db.write(|tx| tx.interrupt_running_ai_journal()).unwrap(), 1);
    let rows = db.read(|tx| tx.list_ai_journal(10, None)).unwrap();
    let outcomes: Vec<&str> = rows.iter().map(|row| row.outcome.as_str()).collect();
    assert_eq!(outcomes, ["interrupted", "end_turn"]);
}

/// Compléter une opération absente du journal donne `not_found.operation`
#[test]
fn finish_unknown_operation_is_not_found() {
    let db = Database::open_in_memory().unwrap();
    let operation = OperationId::from_raw("absente");
    let refused = db.write(|tx| tx.finish_ai_journal(&operation, &finish("end_turn", 0)));
    assert_eq!(refused.unwrap_err().code, "not_found.operation");
}
