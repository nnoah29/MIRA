#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Poses de libellés locaux sur les conversations : pose, retrait, refus et cascades.

mod support;
mod support_mirror;
mod support_reading;

use mira_vocabulary::{AccountId, ThreadId};
use support::local_labels::{label_id, labels_on, store_label};
use support_mirror::seeded::database_with_messages;
use support_reading::threading::conversation;

/// Poser puis retirer un libellé sur un fil ; la pose est un ensemble
#[test]
fn set_and_unset_thread_label() {
    let db = database_with_messages(0);
    let thread = conversation(&db);
    store_label(&db, "l-1", "Urgent");
    for _ in 0..2 {
        db.write(|tx| tx.set_label_on_thread(&label_id("l-1"), &thread, true))
            .unwrap();
    }
    assert_eq!(labels_on(&db, &thread), ["l-1"]);
    db.write(|tx| tx.set_label_on_thread(&label_id("l-1"), &thread, false))
        .unwrap();
    assert!(labels_on(&db, &thread).is_empty());
}

/// Une pose sur un fil inconnu ou avec un libellé inconnu est refusée sans effet
#[test]
fn assignment_on_unknown_thread_is_refused() {
    let db = database_with_messages(0);
    let thread = conversation(&db);
    store_label(&db, "l-1", "Urgent");
    let ghost = ThreadId::from_raw("zz");
    let no_thread = db.write(|tx| tx.set_label_on_thread(&label_id("l-1"), &ghost, true));
    assert_eq!(no_thread.unwrap_err().code, "not_found.thread");
    let no_label = db.write(|tx| tx.set_label_on_thread(&label_id("zz"), &thread, true));
    assert_eq!(no_label.unwrap_err().code, "not_found.local_label");
    assert!(labels_on(&db, &thread).is_empty());
}

/// Supprimer un libellé retire toutes ses poses
#[test]
fn delete_label_removes_its_assignments() {
    let db = database_with_messages(0);
    let thread = conversation(&db);
    store_label(&db, "l-1", "Urgent");
    db.write(|tx| tx.set_label_on_thread(&label_id("l-1"), &thread, true))
        .unwrap();
    db.write(|tx| tx.delete_local_label(&label_id("l-1")))
        .unwrap();
    assert!(labels_on(&db, &thread).is_empty());
    let again = db.write(|tx| tx.delete_local_label(&label_id("l-1")));
    assert_eq!(again.unwrap_err().code, "not_found.local_label");
}

/// Supprimer un compte retire les poses de ses fils, pas le libellé
#[test]
fn deleting_account_removes_assignments() {
    let db = database_with_messages(0);
    let thread = conversation(&db);
    store_label(&db, "l-1", "Urgent");
    db.write(|tx| tx.set_label_on_thread(&label_id("l-1"), &thread, true))
        .unwrap();
    db.write(|tx| tx.delete_account(&AccountId::from_raw("acc-1")))
        .unwrap();
    let rows = db
        .read(|tx| tx.table_row_count("thread_local_labels"))
        .unwrap();
    assert_eq!(rows, 0);
    assert_eq!(db.read(|tx| tx.list_local_labels()).unwrap().len(), 1);
}
