#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Dépôt des apparitions : UID, revalidation, purge, contrôle du compte.

mod support;
mod support_mirror;

use mira_vocabulary::{AccountId, MailboxId, MessageId, Timestamp};
use support_mirror::fixtures::{appearance, sample_message};
use support_mirror::seeded::{database_with_messages, database_with_two_accounts};

/// Invariant 7 : l'identifiant interne survit à un changement d'UID (UIDVALIDITY)
#[test]
fn invariant_7_internal_id_survives_uid_change() {
    let db = database_with_messages(2);
    let mailbox = MailboxId::from_raw("box-1");
    let id = MessageId::from_raw("msg-2");
    db.write(|tx| tx.mark_mailbox_for_revalidation(&mailbox))
        .unwrap();
    let pending = db.read(|tx| tx.appearances_of_message(&id)).unwrap();
    assert!(pending[0].needs_revalidation);
    assert_eq!(pending[0].uid, 2);
    assert!(
        db.read(|tx| tx.uids_in_mailbox(&mailbox))
            .unwrap()
            .is_empty()
    );
    db.write(|tx| tx.revalidate_appearance(&id, &mailbox, 1))
        .unwrap();
    let found = db.read(|tx| tx.appearance_by_uid(&mailbox, 1)).unwrap();
    assert_eq!(found.unwrap().message, id);
    assert!(db.read(|tx| tx.get_message(&id)).unwrap().is_some());
}

/// Invariant 8 : un message sans apparition est purgé, pas les autres
#[test]
fn invariant_8_message_without_appearance_is_purged() {
    let db = database_with_messages(2);
    let mailbox = MailboxId::from_raw("box-1");
    db.write(|tx| tx.remove_appearance(&MessageId::from_raw("msg-1"), &mailbox))
        .unwrap();
    let purged = db
        .write(|tx| tx.delete_orphan_messages(&AccountId::from_raw("acc-1")))
        .unwrap();
    assert_eq!(purged, vec![MessageId::from_raw("msg-1")]);
    assert!(db.read(|tx| tx.get_message(&purged[0])).unwrap().is_none());
    let kept = db.read(|tx| tx.get_message(&MessageId::from_raw("msg-2")));
    assert!(kept.unwrap().is_some());
}

/// Invariant 9 : une apparition dans une boîte d'un autre compte est refusée
#[test]
fn invariant_9_appearance_mailbox_same_account() {
    let db = database_with_two_accounts();
    db.write(|tx| tx.insert_message(&sample_message("msg-1", "acc-1"), Timestamp::from_millis(1)))
        .unwrap();
    let refused = db.write(|tx| tx.add_appearance(&appearance("msg-1", "box-2", 1)));
    assert_eq!(refused.unwrap_err().code, "invalid_input.parameter");
    let unknown = db.write(|tx| tx.add_appearance(&appearance("msg-1", "box-x", 1)));
    assert_eq!(unknown.unwrap_err().code, "not_found.mailbox");
}

/// Les UID d'une boîte se listent, se cherchent, et la marque « supprimé » se bascule
#[test]
fn uids_lookup_and_deleted_flag() {
    let db = database_with_messages(3);
    let mailbox = MailboxId::from_raw("box-1");
    let uids = db.read(|tx| tx.uids_in_mailbox(&mailbox)).unwrap();
    let numbers: Vec<u32> = uids.iter().map(|(uid, _)| *uid).collect();
    assert_eq!(numbers, [1, 2, 3]);
    let id = MessageId::from_raw("msg-2");
    db.write(|tx| tx.set_appearance_deleted(&id, &mailbox, true))
        .unwrap();
    let found = db.read(|tx| tx.appearance_by_uid(&mailbox, 2)).unwrap();
    assert!(found.unwrap().deleted);
    assert!(
        db.read(|tx| tx.appearance_by_uid(&mailbox, 99))
            .unwrap()
            .is_none()
    );
}

/// La purge rend les messages devenus orphelins et épargne ceux présents ailleurs
#[test]
fn purge_unrevalidated_returns_only_orphans() {
    let db = database_with_two_accounts();
    let mailbox = MailboxId::from_raw("box-1");
    db.write(|tx| {
        tx.upsert_mailbox(&support_mirror::fixtures::sample_mailbox(
            "box-3",
            "acc-1",
            "Archive",
            mira_vocabulary::MailboxRole::Archive,
        ))?;
        for name in ["msg-1", "msg-2"] {
            tx.insert_message(&sample_message(name, "acc-1"), Timestamp::from_millis(1))?;
            tx.add_appearance(&appearance(name, "box-1", 1 + u32::from(name == "msg-2")))?;
        }
        tx.add_appearance(&appearance("msg-2", "box-3", 7))?;
        tx.mark_mailbox_for_revalidation(&mailbox)?;
        Ok(())
    })
    .unwrap();
    let orphans = db.write(|tx| tx.purge_unrevalidated(&mailbox)).unwrap();
    assert_eq!(orphans, vec![MessageId::from_raw("msg-1")]);
    let left = db
        .read(|tx| tx.appearances_of_message(&MessageId::from_raw("msg-2")))
        .unwrap();
    assert_eq!(left.len(), 1);
}
