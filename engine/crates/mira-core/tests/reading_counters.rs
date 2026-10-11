#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Compteurs de boîte et drapeaux portés par le message (D-14, invariant 12).

mod support;
mod support_mirror;
mod support_reading;

use mira_core::reading::{list_messages, mailboxes_with_counters};
use mira_vocabulary::{AccountId, MailboxId, MessageId, StandardView};
use support_reading::queries::{profile, standard};
use support_reading::views::{database_with_many_boxes, page_ids, put_message, set_flags};

/// Total et non lus des boîtes box-1 et label-1 du compte acc-1
fn counters(db: &mira_core::storage::Database) -> Vec<(u64, u64)> {
    let boxes = mailboxes_with_counters(db, &AccountId::from_raw("acc-1")).unwrap();
    ["box-1", "label-1"]
        .iter()
        .map(|id| {
            let found = boxes.iter().find(|b| b.id.as_str() == *id).unwrap();
            (found.total, found.unread)
        })
        .collect()
}

/// Invariant 12 : marquer lu une fois change le message, donc toutes ses apparitions et la vue unifiée
#[test]
fn invariant_12_mark_read_applies_to_message_once() {
    let db = database_with_many_boxes();
    put_message(&db, "a", "acc-1", 1_000, &["box-1", "label-1"]);
    put_message(&db, "b", "acc-1", 2_000, &["box-1"]);
    assert_eq!(counters(&db), [(2, 2), (1, 1)]);
    set_flags(&db, "a", true, false);
    assert_eq!(counters(&db), [(2, 1), (1, 0)]);
    let unread = list_messages(&db, &profile(), &standard(StandardView::Unread)).unwrap();
    assert_eq!(page_ids(&unread), ["b"]);
    let unified = list_messages(&db, &profile(), &standard(StandardView::Unified)).unwrap();
    assert_eq!(page_ids(&unified), ["b", "a"]);
    assert!(unified.items[1].flags.seen);
}

/// Les compteurs ignorent les apparitions supprimées et suivent les retraits de boîte
#[test]
fn counters_follow_deleted_and_removed_appearances() {
    let db = database_with_many_boxes();
    put_message(&db, "a", "acc-1", 1_000, &["box-1", "label-1"]);
    put_message(&db, "b", "acc-1", 2_000, &["box-1", "label-1"]);
    let (message, mailbox) = (MessageId::from_raw("a"), MailboxId::from_raw("box-1"));
    db.write(|tx| tx.set_appearance_deleted(&message, &mailbox, true))
        .unwrap();
    assert_eq!(counters(&db), [(1, 1), (2, 2)]);
    let other = MailboxId::from_raw("label-1");
    db.write(|tx| tx.remove_appearance(&message, &other))
        .unwrap();
    assert_eq!(counters(&db), [(1, 1), (1, 1)]);
}
