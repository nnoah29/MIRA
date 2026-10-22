#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Dépôt des libellés locaux : création, unicité du nom, renommage et couleur.

mod support;
mod support_mirror;

use support::local_labels::{label, label_id, store_label};
use support_mirror::seeded::database_with_messages;

/// Un libellé inséré se relit à l'identique et figure dans la liste
#[test]
fn create_get_list_roundtrip() {
    let db = database_with_messages(0);
    let urgent = label("l-1", "Urgent");
    db.write(|tx| tx.insert_local_label(&urgent)).unwrap();
    let loaded = db.read(|tx| tx.get_local_label(&urgent.id)).unwrap();
    assert_eq!(loaded, Some(urgent.clone()));
    assert_eq!(db.read(|tx| tx.list_local_labels()).unwrap(), [urgent]);
    let unknown = db.read(|tx| tx.get_local_label(&label_id("zz"))).unwrap();
    assert_eq!(unknown, None);
}

/// Deux noms qui ne diffèrent que par la casse (accents compris) sont un doublon
#[test]
fn duplicate_name_ignoring_case_is_refused() {
    let db = database_with_messages(0);
    store_label(&db, "l-1", "À payer");
    let second = db.write(|tx| tx.insert_local_label(&label("l-2", "à PAYER")));
    assert!(second.is_err());
    assert_eq!(db.read(|tx| tx.list_local_labels()).unwrap().len(), 1);
}

/// Renommer et recolorer changent le libellé ; un nom déjà pris ou inconnu est refusé
#[test]
fn rename_and_recolor() {
    let db = database_with_messages(0);
    store_label(&db, "l-1", "Un");
    store_label(&db, "l-2", "Deux");
    db.write(|tx| tx.rename_local_label(&label_id("l-1"), "Trois"))
        .unwrap();
    db.write(|tx| tx.recolor_local_label(&label_id("l-1"), 0xFF00_FF00))
        .unwrap();
    let renamed = db
        .read(|tx| tx.get_local_label(&label_id("l-1")))
        .unwrap()
        .unwrap();
    assert_eq!(
        (renamed.name.as_str(), renamed.color),
        ("Trois", 0xFF00_FF00)
    );
    let taken = db.write(|tx| tx.rename_local_label(&label_id("l-1"), "deux"));
    assert!(taken.is_err());
    let unknown = db.write(|tx| tx.rename_local_label(&label_id("zz"), "X"));
    assert_eq!(unknown.unwrap_err().code, "not_found.local_label");
}
