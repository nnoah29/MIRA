#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Dépôt des boîtes : upsert, recherche, repère de synchronisation, compteurs.

mod support;
mod support_mirror;

use mira_vocabulary::{AccountId, MailboxId, MailboxRole, MessageId, SyncMarker, Timestamp};
use support_mirror::fixtures::{appearance, sample_mailbox, sample_message};
use support_mirror::seeded::database_with_two_accounts;

/// Un upsert sur le même (compte, nom) garde l'identifiant existant et met à jour le reste
#[test]
fn upsert_mailbox_keeps_existing_id() {
    let db = database_with_two_accounts();
    let mut renamed = sample_mailbox("box-other", "acc-1", "INBOX", MailboxRole::Inbox);
    renamed.display_name = "Réception".to_string();
    let id = db.write(|tx| tx.upsert_mailbox(&renamed)).unwrap();
    assert_eq!(id, MailboxId::from_raw("box-1"));
    let loaded = db.read(|tx| tx.get_mailbox(&id)).unwrap().unwrap();
    assert_eq!(loaded.display_name, "Réception");
    let all = db
        .read(|tx| tx.list_mailboxes(&AccountId::from_raw("acc-1")))
        .unwrap();
    assert_eq!(all.len(), 1);
}

/// Les boîtes se retrouvent par rôle et par nom, dans leur compte seulement
#[test]
fn mailboxes_are_found_by_role_and_name() {
    let db = database_with_two_accounts();
    let acc = AccountId::from_raw("acc-1");
    db.write(|tx| {
        tx.upsert_mailbox(&sample_mailbox("box-s", "acc-1", "Sent", MailboxRole::Sent))
            .map(|_| ())
    })
    .unwrap();
    let sent = db
        .read(|tx| tx.find_mailbox_by_role(&acc, MailboxRole::Sent))
        .unwrap();
    assert_eq!(sent.unwrap().id, MailboxId::from_raw("box-s"));
    let inbox = db
        .read(|tx| tx.find_mailbox_by_name(&acc, "INBOX"))
        .unwrap();
    assert_eq!(inbox.unwrap().id, MailboxId::from_raw("box-1"));
    let none = db
        .read(|tx| tx.find_mailbox_by_role(&acc, MailboxRole::Trash))
        .unwrap();
    assert!(none.is_none());
}

/// Le repère de synchronisation s'enregistre, se relit et s'efface
#[test]
fn mailbox_marker_roundtrips_and_clears() {
    let db = database_with_two_accounts();
    let id = MailboxId::from_raw("box-1");
    let marker = SyncMarker {
        uid_validity: 77,
        uid_next: 12,
        highest_modseq: Some(900),
    };
    db.write(|tx| tx.set_mailbox_marker(&id, Some(marker)))
        .unwrap();
    let loaded = db.read(|tx| tx.get_mailbox(&id)).unwrap().unwrap();
    assert_eq!(loaded.marker, Some(marker));
    db.write(|tx| tx.set_mailbox_marker(&id, None)).unwrap();
    let cleared = db.read(|tx| tx.get_mailbox(&id)).unwrap().unwrap();
    assert_eq!(cleared.marker, None);
}

/// Les compteurs viennent des apparitions non supprimées et des messages non lus
#[test]
fn mailbox_counters_are_computed_from_appearances() {
    let db = database_with_two_accounts();
    let id = MailboxId::from_raw("box-1");
    db.write(|tx| {
        for index in 1..=3_u32 {
            let name = format!("m{index}");
            let mut message = sample_message(&name, "acc-1");
            message.flags.seen = index == 1;
            tx.insert_message(&message, Timestamp::from_millis(i64::from(index)))?;
            tx.add_appearance(&appearance(&name, "box-1", index))?;
        }
        tx.set_appearance_deleted(&MessageId::from_raw("m3"), &id, true)?;
        tx.refresh_mailbox_counters(&id)
    })
    .unwrap();
    let loaded = db.read(|tx| tx.get_mailbox(&id)).unwrap().unwrap();
    assert_eq!((loaded.total, loaded.unread), (2, 1));
}

/// Supprimer une boîte la retire, et une boîte inconnue est signalée par `not_found.mailbox`
#[test]
fn delete_mailbox_removes_it_and_unknown_is_reported() {
    let db = database_with_two_accounts();
    let id = MailboxId::from_raw("box-1");
    db.write(|tx| tx.delete_mailbox(&id)).unwrap();
    assert!(db.read(|tx| tx.get_mailbox(&id)).unwrap().is_none());
    let error = db.write(|tx| tx.refresh_mailbox_counters(&id)).unwrap_err();
    assert_eq!(error.code, "not_found.mailbox");
}
