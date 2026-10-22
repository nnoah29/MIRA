#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Poses de libellés locaux et fils recalculés : recalcul, rejet du miroir, liste des poses.

mod support;
mod support_mirror;
mod support_reading;

use mira_core::ports::SequentialIds;
use mira_core::threads::recompute_threads;
use mira_vocabulary::AccountId;
use support::local_labels::{label_id, labels_on, store_label};
use support_mirror::seeded::database_with_messages;
use support_reading::threading::{conversation, thread_of};

/// Recalculer les fils d'un compte conserve les poses (la pose suit la clé du fil)
#[test]
fn assignments_survive_recompute_threads() {
    let db = database_with_messages(0);
    let thread = conversation(&db);
    store_label(&db, "l-1", "Urgent");
    db.write(|tx| tx.set_label_on_thread(&label_id("l-1"), &thread, true))
        .unwrap();
    let account = AccountId::from_raw("acc-1");
    recompute_threads(&db, &SequentialIds::new(), &account).unwrap();
    let after = thread_of(&db, "a").unwrap();
    assert_eq!(labels_on(&db, &after), ["l-1"]);
}

/// Jeter le miroir puis resynchroniser redonne les poses : mêmes en-têtes, même clé de fil
#[test]
fn assignments_survive_mirror_discard_and_resync() {
    let db = database_with_messages(0);
    let thread = conversation(&db);
    store_label(&db, "l-1", "Urgent");
    db.write(|tx| tx.set_label_on_thread(&label_id("l-1"), &thread, true))
        .unwrap();
    db.write(|tx| tx.discard_account_mirror(&AccountId::from_raw("acc-1")))
        .unwrap();
    let again = conversation(&db);
    assert_eq!(labels_on(&db, &again), ["l-1"]);
}

/// Seules les poses de fils qui existent encore sont listées, groupées par fil
#[test]
fn list_assignments_only_returns_existing_threads() {
    let db = database_with_messages(0);
    let thread = conversation(&db);
    store_label(&db, "l-1", "Urgent");
    store_label(&db, "l-2", "Banque");
    for l in ["l-1", "l-2"] {
        db.write(|tx| tx.set_label_on_thread(&label_id(l), &thread, true))
            .unwrap();
    }
    let listed = db.read(|tx| tx.list_label_assignments()).unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].thread, thread);
    assert_eq!(listed[0].labels.len(), 2);
    db.write(|tx| tx.discard_account_mirror(&AccountId::from_raw("acc-1")))
        .unwrap();
    assert!(
        db.read(|tx| tx.list_label_assignments())
            .unwrap()
            .is_empty()
    );
}
