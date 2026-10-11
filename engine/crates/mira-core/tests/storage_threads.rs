#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Dépôt des fils (table dérivée) : création idempotente, liste par boîte, effacement.

mod support;
mod support_mirror;

use mira_vocabulary::{AccountId, MailboxId, MessageId, PageRequest, ThreadId};
use support_mirror::seeded::database_with_messages;
use support_mirror::threads::database_with_three_threads;

/// Deux créations de la même clé de racine donnent le même fil
#[test]
fn upsert_thread_is_idempotent_per_root_key() {
    let db = database_with_messages(0);
    let acc = AccountId::from_raw("acc-1");
    let first = db
        .write(|tx| tx.upsert_thread(&acc, "root-1", "Sujet", &ThreadId::from_raw("t-1")))
        .unwrap();
    let again = db
        .write(|tx| tx.upsert_thread(&acc, "root-1", "Autre", &ThreadId::from_raw("t-2")))
        .unwrap();
    assert_eq!(first, again);
    assert_eq!(first, ThreadId::from_raw("t-1"));
    let other = db
        .write(|tx| tx.upsert_thread(&acc, "root-2", "Sujet", &ThreadId::from_raw("t-3")))
        .unwrap();
    assert_eq!(other, ThreadId::from_raw("t-3"));
}

/// Un fil expose ses compteurs, ses participants et ses messages du plus ancien au plus récent
#[test]
fn thread_has_counters_participants_and_messages() {
    let db = database_with_messages(3);
    let thread = ThreadId::from_raw("t-1");
    db.write(|tx| {
        tx.upsert_thread(&AccountId::from_raw("acc-1"), "root", "Sujet", &thread)?;
        for name in ["msg-1", "msg-3"] {
            tx.set_thread(&MessageId::from_raw(name), Some(&thread))?;
        }
        Ok(())
    })
    .unwrap();
    let loaded = db.read(|tx| tx.get_thread(&thread)).unwrap().unwrap();
    assert_eq!((loaded.message_count, loaded.unread_count), (2, 2));
    assert_eq!(loaded.last_date.unwrap().as_millis(), 3_000);
    assert_eq!(loaded.participants.len(), 1);
    let messages = db.read(|tx| tx.messages_of_thread(&thread)).unwrap();
    let ids: Vec<&str> = messages.iter().map(|m| m.id.as_str()).collect();
    assert_eq!(ids, ["msg-1", "msg-3"]);
}

/// Les fils d'une boîte sont paginés par activité récente
#[test]
fn threads_in_mailbox_are_paginated_by_recent_activity() {
    let db = database_with_three_threads();
    let mailbox = MailboxId::from_raw("box-1");
    let request = PageRequest {
        cursor: None,
        limit: 2,
    };
    let first = db
        .read(|tx| tx.list_threads_in_mailbox(&mailbox, &request))
        .unwrap();
    let ids: Vec<&str> = first.items.iter().map(|t| t.id.as_str()).collect();
    assert_eq!(ids, ["t-2", "t-1"]);
    let next = PageRequest {
        cursor: first.next_cursor,
        limit: 2,
    };
    let second = db
        .read(|tx| tx.list_threads_in_mailbox(&mailbox, &next))
        .unwrap();
    assert_eq!(second.items.len(), 1);
    assert!(second.next_cursor.is_none());
}

/// Effacer les fils détache les messages et supprime les fils
#[test]
fn clear_threads_detaches_messages_and_removes_threads() {
    let db = database_with_three_threads();
    db.write(|tx| tx.clear_threads(&AccountId::from_raw("acc-1")))
        .unwrap();
    let message = db
        .read(|tx| tx.get_message(&MessageId::from_raw("msg-1")))
        .unwrap();
    assert_eq!(message.unwrap().thread, None);
    let gone = db.read(|tx| tx.get_thread(&ThreadId::from_raw("t-0")));
    assert!(gone.unwrap().is_none());
}
