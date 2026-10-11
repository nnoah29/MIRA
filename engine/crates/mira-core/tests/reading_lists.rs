#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Listes par vue (7.10) : boîte, unifiée (S11), non lus, suivis, pagination (invariant 29).

mod support;
mod support_mirror;
mod support_reading;

use mira_core::reading::{ListQuery, list_messages};
use mira_vocabulary::{AccountId, MailboxId, MessageId, PageRequest, StandardView, ViewRef};
use support_reading::queries::{profile, standard};
use support_reading::views::{database_with_many_boxes, page_ids, put_message, set_flags};

/// Une boîte liste ses messages du plus récent au plus ancien, sans les apparitions supprimées
#[test]
fn mailbox_view_lists_visible_messages_newest_first() {
    let db = database_with_many_boxes();
    put_message(&db, "m1", "acc-1", 1_000, &["box-1"]);
    put_message(&db, "m2", "acc-1", 2_000, &["box-1"]);
    put_message(&db, "m3", "acc-1", 3_000, &["box-1"]);
    put_message(&db, "m4", "acc-1", 4_000, &["sent-1"]);
    let (mailbox, message) = (MailboxId::from_raw("box-1"), MessageId::from_raw("m3"));
    db.write(|tx| tx.set_appearance_deleted(&message, &mailbox, true))
        .unwrap();
    let query = ListQuery::of(ViewRef::Mailbox { mailbox });
    let page = list_messages(&db, &profile(), &query).unwrap();
    assert_eq!(page_ids(&page), ["m2", "m1"]);
}

/// S11 : la vue unifiée prend les réceptions de tous les comptes et chaque message n'y figure qu'une fois
#[test]
fn scenario_s11_unified_view_lists_each_message_once() {
    let db = database_with_many_boxes();
    put_message(&db, "a", "acc-1", 1_000, &["box-1", "box-1b"]);
    put_message(&db, "b", "acc-2", 2_000, &["box-2"]);
    put_message(&db, "c", "acc-1", 3_000, &["sent-1"]);
    put_message(&db, "d", "acc-1", 4_000, &["label-1", "box-1"]);
    let page = list_messages(&db, &profile(), &standard(StandardView::Unified)).unwrap();
    assert_eq!(page_ids(&page), ["d", "b", "a"]);
    let only_first = [AccountId::from_raw("acc-1")];
    let page = list_messages(&db, &only_first, &standard(StandardView::Unified)).unwrap();
    assert_eq!(page_ids(&page), ["d", "a"]);
    let none = list_messages(&db, &[], &standard(StandardView::Unified)).unwrap();
    assert!(none.items.is_empty());
}

/// Les vues « non lus » et « suivis » suivent les drapeaux du message, tous comptes confondus
#[test]
fn unread_and_flagged_views_follow_message_flags() {
    let db = database_with_many_boxes();
    put_message(&db, "a", "acc-1", 1_000, &["box-1"]);
    put_message(&db, "b", "acc-2", 2_000, &["box-2"]);
    put_message(&db, "c", "acc-1", 3_000, &["sent-1", "label-1"]);
    set_flags(&db, "a", true, true);
    set_flags(&db, "b", false, false);
    set_flags(&db, "c", false, true);
    let unread = list_messages(&db, &profile(), &standard(StandardView::Unread)).unwrap();
    assert_eq!(page_ids(&unread), ["c", "b"]);
    let flagged = list_messages(&db, &profile(), &standard(StandardView::Flagged)).unwrap();
    assert_eq!(page_ids(&flagged), ["c", "a"]);
}

/// Invariant 29 : la vue unifiée se parcourt page par page sans doublon ni oubli
#[test]
fn invariant_29_unified_view_paginates_without_duplicates() {
    let db = database_with_many_boxes();
    for index in 1..=25_i64 {
        let (account, boxes): (_, &[&str]) = match index % 3 {
            0 => ("acc-2", &["box-2"]),
            1 => ("acc-1", &["box-1", "box-1b"]),
            _ => ("acc-1", &["box-1"]),
        };
        put_message(&db, &format!("m{index:02}"), account, index * 1_000, boxes);
    }
    let mut query = standard(StandardView::Unified);
    query.page = PageRequest {
        cursor: None,
        limit: 7,
    };
    let mut seen = Vec::new();
    loop {
        let page = list_messages(&db, &profile(), &query).unwrap();
        assert!(page.items.len() <= 7);
        seen.extend(page_ids(&page));
        match page.next_cursor {
            Some(cursor) => query.page.cursor = Some(cursor),
            None => break,
        }
    }
    let expected: Vec<String> = (1..=25).rev().map(|i| format!("m{i:02}")).collect();
    assert_eq!(seen, expected);
}

/// La limite demandée est bornée à 500 et un curseur invalide est refusé
#[test]
fn page_limit_is_bounded_and_bad_cursor_refused() {
    let db = database_with_many_boxes();
    for index in 1..=520_i64 {
        put_message(
            &db,
            &format!("m{index}"),
            "acc-1",
            index * 1_000,
            &["box-1"],
        );
    }
    let mut query = standard(StandardView::Unified);
    query.page = PageRequest {
        cursor: None,
        limit: 100_000,
    };
    let page = list_messages(&db, &profile(), &query).unwrap();
    assert_eq!(page.items.len(), 500);
    assert!(page.next_cursor.is_some());
    query.page.cursor = Some(mira_vocabulary::Cursor("zz".to_string()));
    let error = list_messages(&db, &profile(), &query).unwrap_err();
    assert_eq!(error.code, "invalid_input.cursor");
}
