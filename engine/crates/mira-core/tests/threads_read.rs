#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Lectures locales des fils : liste d'une boîte, contenu d'un fil, fil inconnu.

mod support;
mod support_mirror;
mod support_reading;

use mira_core::threads::{get_thread, list_threads};
use mira_vocabulary::{MailboxId, PageRequest, ThreadId};
use support_mirror::fixtures::appearance;
use support_mirror::seeded::database_with_messages;
use support_reading::threading::{store_and_assign, threaded};

/// Deux conversations de la boîte box-1 : trois messages, puis un message isolé plus récent
fn two_conversations() -> mira_core::storage::Database {
    let db = database_with_messages(0);
    let messages = [
        threaded("root", "Plan", Some("<root>"), None, &[]),
        threaded("a", "Re: Plan", Some("<a>"), Some("<root>"), &["<root>"]),
        threaded(
            "b",
            "Re: Plan",
            Some("<b>"),
            Some("<a>"),
            &["<root>", "<a>"],
        ),
        threaded("solo", "Solo", Some("<solo>"), None, &[]),
    ];
    for (index, message) in messages.iter().enumerate() {
        let uid = u32::try_from(index).unwrap() + 1;
        store_and_assign(&db, message, 1_000 * i64::from(uid), None);
        let seen = appearance(message.id.as_str(), "box-1", uid);
        db.write(|tx| tx.add_appearance(&seen)).unwrap();
    }
    db
}

/// La liste des fils d'une boîte va du plus récemment actif au plus ancien, avec compteurs
#[test]
fn list_threads_orders_by_recent_activity() {
    let db = two_conversations();
    let page = list_threads(&db, &MailboxId::from_raw("box-1"), &PageRequest::default()).unwrap();
    let subjects: Vec<&str> = page.items.iter().map(|t| t.subject.as_str()).collect();
    assert_eq!(subjects, ["Solo", "Plan"]);
    assert_eq!(page.items[1].message_count, 3);
    assert_eq!(page.items[1].unread_count, 3);
}

/// Un fil rend ses messages du plus ancien au plus récent, avec son objet sans préfixe
#[test]
fn get_thread_returns_messages_oldest_first() {
    let db = two_conversations();
    let page = list_threads(&db, &MailboxId::from_raw("box-1"), &PageRequest::default()).unwrap();
    let (thread, messages) = get_thread(&db, &page.items[1].id).unwrap();
    assert_eq!(thread.subject, "Plan");
    let ids: Vec<&str> = messages.iter().map(|m| m.id.as_str()).collect();
    assert_eq!(ids, ["root", "a", "b"]);
}

/// Un fil inconnu est une erreur « not_found.thread »
#[test]
fn unknown_thread_is_not_found() {
    let db = two_conversations();
    let error = get_thread(&db, &ThreadId::from_raw("nope")).unwrap_err();
    assert_eq!(error.code, "not_found.thread");
}
