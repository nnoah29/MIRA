#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Vue d'un libellé local : ses messages restent listés même masqués de la Réception.

mod support;
mod support_mirror;
mod support_reading;

use mira_core::reading::{ListQuery, list_messages};
use mira_vocabulary::{AccountId, LabelKind, LabelRef, LabelSettings, StandardView, ViewRef};
use support::local_labels::{label_id, store_label};
use support_mirror::fixtures::appearance;
use support_mirror::seeded::database_with_messages;
use support_reading::queries::{profile, standard};
use support_reading::threading::conversation;
use support_reading::views::page_ids;

/// Le libellé liste les messages de son fil, et seulement ceux-là
#[test]
fn local_label_view_lists_the_messages_of_labelled_threads() {
    let db = database_with_messages(0);
    let thread = inbox_conversation(&db);
    store_label(&db, "l-1", "Urgent");
    store_label(&db, "l-2", "Banque");
    db.write(|tx| tx.set_label_on_thread(&label_id("l-1"), &thread, true))
        .unwrap();
    let urgent = ListQuery::of(ViewRef::LocalLabel {
        label: label_id("l-1"),
    });
    let bank = ListQuery::of(ViewRef::LocalLabel {
        label: label_id("l-2"),
    });
    assert!(!page_ids(&list_messages(&db, &profile(), &urgent).unwrap()).is_empty());
    assert!(
        list_messages(&db, &profile(), &bank)
            .unwrap()
            .items
            .is_empty()
    );
}

/// Masqué de la Réception, le message reste dans la vue de son libellé
#[test]
fn hidden_from_inbox_messages_stay_in_the_label_view() {
    let db = database_with_messages(0);
    let thread = inbox_conversation(&db);
    store_label(&db, "l-1", "Urgent");
    db.write(|tx| tx.set_label_on_thread(&label_id("l-1"), &thread, true))
        .unwrap();
    hide_from_inbox(&db, "l-1");
    let query = ListQuery::of(ViewRef::LocalLabel {
        label: label_id("l-1"),
    });
    let page = list_messages(&db, &profile(), &query).unwrap();
    assert!(!page.items.is_empty());
    let inbox = list_messages(&db, &profile(), &standard(StandardView::Unified)).unwrap();
    assert!(inbox.items.is_empty());
    let none: [AccountId; 0] = [];
    assert!(list_messages(&db, &none, &query).unwrap().items.is_empty());
}

fn hide_from_inbox(db: &mira_core::storage::Database, id: &str) {
    let label = LabelRef {
        kind: LabelKind::Local,
        id: id.to_string(),
    };
    let settings = LabelSettings {
        hide_from_inbox: true,
        ..LabelSettings::default()
    };
    db.write(|tx| tx.put_label_settings(&label, &settings))
        .unwrap();
}

fn inbox_conversation(db: &mira_core::storage::Database) -> mira_vocabulary::ThreadId {
    let thread = conversation(db);
    db.write(|tx| {
        tx.add_appearance(&appearance("root", "box-1", 1))?;
        tx.add_appearance(&appearance("a", "box-1", 2))
    })
    .unwrap();
    thread
}
