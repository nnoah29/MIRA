#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Dépôt des messages : aller-retour, drapeaux, pagination par curseur.

mod support;
mod support_mirror;

use mira_vocabulary::{
    ContentState, Cursor, Flags, MailboxId, MessageId, PageRequest, ThreadId, Timestamp,
};
use support_mirror::fixtures::{appearance, sample_message};
use support_mirror::seeded::{database_with_messages, database_with_two_accounts};

/// Un message relu est identique à l'inséré, avec les boîtes tirées de ses apparitions
#[test]
fn message_roundtrips_with_mailboxes_from_appearances() {
    let db = database_with_two_accounts();
    let mut message = sample_message("msg-1", "acc-1");
    db.write(|tx| {
        tx.insert_message(&message, Timestamp::from_millis(9_000))?;
        tx.add_appearance(&appearance("msg-1", "box-1", 4))
    })
    .unwrap();
    message.mailboxes = vec![MailboxId::from_raw("box-1")];
    let loaded = db.read(|tx| tx.get_message(&message.id)).unwrap();
    assert_eq!(loaded, Some(message));
    let unknown = db
        .read(|tx| tx.get_message(&MessageId::from_raw("nope")))
        .unwrap();
    assert!(unknown.is_none());
}

/// Invariant 12 : les drapeaux vivent sur le message et valent pour toutes ses boîtes
#[test]
fn invariant_12_flags_live_on_message() {
    let db = database_with_messages(1);
    let id = MessageId::from_raw("msg-1");
    let flags = Flags {
        seen: true,
        flagged: true,
        answered: false,
        forwarded: true,
    };
    db.write(|tx| tx.update_message_flags(&id, flags)).unwrap();
    let loaded = db.read(|tx| tx.get_message(&id)).unwrap().unwrap();
    assert_eq!(loaded.flags, flags);
    let missing = db.write(|tx| tx.update_message_flags(&MessageId::from_raw("zz"), flags));
    assert_eq!(missing.unwrap_err().code, "not_found.message");
}

/// L'état du contenu et le fil se modifient, puis le message se supprime
#[test]
fn content_state_thread_and_delete() {
    let db = database_with_messages(1);
    let id = MessageId::from_raw("msg-1");
    db.write(|tx| {
        tx.set_content_state(&id, ContentState::BodyPresent)?;
        tx.set_thread(&id, Some(&ThreadId::from_raw("t-1")))
    })
    .unwrap();
    let loaded = db.read(|tx| tx.get_message(&id)).unwrap().unwrap();
    assert_eq!(loaded.content, ContentState::BodyPresent);
    assert_eq!(loaded.thread, Some(ThreadId::from_raw("t-1")));
    db.write(|tx| tx.delete_message(&id)).unwrap();
    assert!(db.read(|tx| tx.get_message(&id)).unwrap().is_none());
}

/// Invariant 29 : 5 messages en 3 pages de 2, sans doublon ni trou, du plus récent au plus ancien
#[test]
fn invariant_29_list_is_cursor_paginated() {
    let db = database_with_messages(5);
    let mailbox = MailboxId::from_raw("box-1");
    let mut request = PageRequest {
        cursor: None,
        limit: 2,
    };
    let mut seen = Vec::new();
    let mut pages = 0;
    loop {
        let page = db
            .read(|tx| tx.list_messages_in_mailbox(&mailbox, &request))
            .unwrap();
        pages += 1;
        seen.extend(page.items.iter().map(|m| m.id.as_str().to_string()));
        match page.next_cursor {
            Some(cursor) => request.cursor = Some(cursor),
            None => break,
        }
    }
    assert_eq!(pages, 3);
    assert_eq!(seen, ["msg-5", "msg-4", "msg-3", "msg-2", "msg-1"]);
}

/// Un curseur fabriqué à la main est refusé par `invalid_input.cursor`
#[test]
fn invalid_cursor_is_rejected() {
    let db = database_with_messages(2);
    let request = PageRequest {
        cursor: Some(Cursor("5000:msg-1".to_string())),
        limit: 2,
    };
    let result = db.read(|tx| tx.list_messages_in_mailbox(&MailboxId::from_raw("box-1"), &request));
    assert_eq!(result.unwrap_err().code, "invalid_input.cursor");
}

/// Les messages marqués supprimés sur le serveur ne sont pas listés
#[test]
fn deleted_appearances_are_not_listed() {
    let db = database_with_messages(3);
    let mailbox = MailboxId::from_raw("box-1");
    db.write(|tx| tx.set_appearance_deleted(&MessageId::from_raw("msg-2"), &mailbox, true))
        .unwrap();
    let page = db
        .read(|tx| tx.list_messages_in_mailbox(&mailbox, &PageRequest::default()))
        .unwrap();
    let ids: Vec<&str> = page.items.iter().map(|m| m.id.as_str()).collect();
    assert_eq!(ids, ["msg-3", "msg-1"]);
}
