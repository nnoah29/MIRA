//! Contenu du monde de test : comptes, identité, boîtes, messages, identifiant stable, repère d'UID.

use mira_core::storage::Database;
use mira_core::storage::repositories::server_ids::ServerIdKind;
use mira_vocabulary::{AccountId, MailboxId, MailboxRole, MessageId, SyncMarker, Timestamp};

use crate::support::accounts::{sample_account, sample_identity};
use crate::support_mirror::fixtures::{appearance, sample_mailbox, sample_message};

const ACCOUNT_BOXES: [(&str, &str, MailboxRole); 7] = [
    ("box-in", "INBOX", MailboxRole::Inbox),
    ("box-arch", "Archive", MailboxRole::Archive),
    ("box-trash", "Trash", MailboxRole::Trash),
    ("box-junk", "Junk", MailboxRole::Junk),
    ("box-sent", "Sent", MailboxRole::Sent),
    ("box-proj", "Projects", MailboxRole::Other),
    ("box-work", "Work", MailboxRole::Other),
];

/// Remplit la base : acc-1 (identité `main`), acc-2, leurs boîtes et leurs messages
pub fn seed_world(db: &Database) {
    db.write(|tx| {
        tx.insert_account(&sample_account("acc-1"))?;
        tx.insert_account(&sample_account("acc-2"))?;
        tx.insert_identity(&sample_identity("main", "acc-1", true))?;
        for (id, name, role) in ACCOUNT_BOXES {
            tx.upsert_mailbox(&sample_mailbox(id, "acc-1", name, role))?;
        }
        tx.upsert_mailbox(&sample_mailbox(
            "box-2-in",
            "acc-2",
            "INBOX",
            MailboxRole::Inbox,
        ))?;
        Ok(())
    })
    .unwrap();
    seed_messages(db);
}

/// msg-1 et msg-2 dans la réception, msg-3 dans la réception et « Work », msg-other chez acc-2
fn seed_messages(db: &Database) {
    db.write(|tx| {
        for (index, id) in ["msg-1", "msg-2", "msg-3"].into_iter().enumerate() {
            let date = Timestamp::from_millis(1_000 * (index as i64 + 1));
            tx.insert_message(&sample_message(id, "acc-1"), date)?;
            tx.add_appearance(&appearance(id, "box-in", index as u32 + 1))?;
        }
        tx.add_appearance(&appearance("msg-3", "box-work", 1))?;
        tx.insert_message(
            &sample_message("msg-other", "acc-2"),
            Timestamp::from_millis(500),
        )?;
        tx.add_appearance(&appearance("msg-other", "box-2-in", 1))
    })
    .unwrap();
    seed_server_state(db);
}

/// Identifiant stable de msg-1, validité d'UID de la réception, compteurs à jour
fn seed_server_state(db: &Database) {
    db.write(|tx| {
        let account = AccountId::from_raw("acc-1");
        let stable = MessageId::from_raw("msg-1");
        tx.map_server_id(&account, ServerIdKind::Stable, "gm-1", &stable)?;
        let marker = SyncMarker {
            uid_validity: 77,
            uid_next: 10,
            highest_modseq: None,
        };
        tx.set_mailbox_marker(&MailboxId::from_raw("box-in"), Some(marker))?;
        for (id, ..) in ACCOUNT_BOXES {
            tx.refresh_mailbox_counters(&MailboxId::from_raw(id))?;
        }
        Ok(())
    })
    .unwrap();
}
