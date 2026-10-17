#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Filtres de liste : lus, suivis, pièces jointes, expéditeur, période.

mod support;
mod support_mirror;
mod support_reading;

use mira_core::reading::{ListQuery, MessageFilters, list_messages};
use mira_core::storage::Database;
use mira_vocabulary::{MailboxId, StandardView, Timestamp, ViewRef};
use support_mirror::fixtures::sample_message;
use support_reading::queries::{profile, standard};
use support_reading::views::{
    database_with_many_boxes, page_ids, put_custom, put_message, sender, set_flags,
};

/// Ids rendus par la vue unifiée avec les filtres donnés
fn unified_with(db: &Database, filters: MessageFilters) -> Vec<String> {
    let mut query = standard(StandardView::Unified);
    query.filters = filters;
    page_ids(&list_messages(db, &profile(), &query).unwrap())
}

/// Les filtres « non lus », « suivis » et « avec pièce jointe » se combinent par « et »
#[test]
fn flag_and_attachment_filters_combine() {
    let db = database_with_many_boxes();
    for (id, at) in [("a", 1_000), ("b", 2_000), ("c", 3_000)] {
        put_message(&db, id, "acc-1", at, &["box-1"]);
    }
    set_flags(&db, "a", true, true);
    set_flags(&db, "b", false, true);
    let mut with_file = sample_message("d", "acc-1");
    with_file.has_attachments = true;
    put_custom(&db, &with_file, 4_000, &["box-1"]);
    let unread = MessageFilters {
        unread_only: true,
        ..MessageFilters::default()
    };
    assert_eq!(unified_with(&db, unread.clone()), ["d", "c", "b"]);
    let flagged = MessageFilters {
        flagged_only: true,
        ..unread.clone()
    };
    assert_eq!(unified_with(&db, flagged), ["b"]);
    let attached = MessageFilters {
        has_attachments: true,
        ..unread
    };
    assert_eq!(unified_with(&db, attached), ["d"]);
}

/// L'expéditeur est cherché dans le nom et l'adresse, sans casse ; `%` et `_` sont des caractères ordinaires
#[test]
fn from_contains_matches_name_or_address_literally() {
    let db = database_with_many_boxes();
    for (id, name, address, at) in [
        ("a", "Zoé Martin", "zoe@example.org", 1_000),
        ("b", "Bob", "bob_100%@example.org", 2_000),
        ("c", "Bobby", "bobby@example.org", 3_000),
    ] {
        let mut message = sample_message(id, "acc-1");
        message.from = sender(name, address);
        put_custom(&db, &message, at, &["box-1"]);
    }
    let from = |text: &str| MessageFilters {
        from_contains: Some(text.to_string()),
        ..MessageFilters::default()
    };
    assert_eq!(unified_with(&db, from("ZOE@")), ["a"]);
    assert_eq!(unified_with(&db, from("example.org")), ["c", "b", "a"]);
    assert_eq!(unified_with(&db, from("_100%")), ["b"]);
    assert_eq!(unified_with(&db, from("%")), ["b"]);
    assert!(unified_with(&db, from("name")).is_empty());
}

/// La période est inclusive à ses deux bornes
#[test]
fn period_filter_is_inclusive_on_both_bounds() {
    let db = database_with_many_boxes();
    for index in 1..=5_i64 {
        put_message(
            &db,
            &format!("m{index}"),
            "acc-1",
            index * 1_000,
            &["box-1"],
        );
    }
    let period = MessageFilters {
        since: Some(Timestamp::from_millis(2_000)),
        until: Some(Timestamp::from_millis(4_000)),
        ..MessageFilters::default()
    };
    assert_eq!(unified_with(&db, period), ["m4", "m3", "m2"]);
}

/// Les filtres s'appliquent aussi à la vue d'une boîte
#[test]
fn filters_apply_to_mailbox_views() {
    let db = database_with_many_boxes();
    put_message(&db, "a", "acc-1", 1_000, &["label-1"]);
    put_message(&db, "b", "acc-1", 2_000, &["label-1"]);
    put_message(&db, "c", "acc-1", 3_000, &["box-1"]);
    set_flags(&db, "a", true, false);
    let mailbox = MailboxId::from_raw("label-1");
    let mut query = ListQuery::of(ViewRef::Mailbox { mailbox });
    query.filters.unread_only = true;
    let page = list_messages(&db, &profile(), &query).unwrap();
    assert_eq!(page_ids(&page), ["b"]);
}
