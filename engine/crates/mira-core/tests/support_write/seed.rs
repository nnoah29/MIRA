//! Contenu du faux serveur et du miroir : mêmes boîtes, mêmes messages, mêmes UID.

use mira_core::storage::Database;
use mira_testkit::imap::FakeMailStore;
use mira_vocabulary::{AccountId, MailboxId, MailboxRole, SyncMarker, Timestamp};

use crate::support_imap::mail::mail;
use crate::support_mirror::fixtures::{appearance, sample_mailbox, sample_message};

const BOXES: [(&str, &str, MailboxRole); 7] = [
    ("box-in", "INBOX", MailboxRole::Inbox),
    ("box-arch", "Archive", MailboxRole::Archive),
    ("box-trash", "Trash", MailboxRole::Trash),
    ("box-junk", "Junk", MailboxRole::Junk),
    ("box-sent", "Sent", MailboxRole::Sent),
    ("box-drafts", "Drafts", MailboxRole::Drafts),
    ("box-work", "Work", MailboxRole::Other),
];

/// Crée les boîtes du serveur ; msg-1 à msg-3 dans la réception, msg-3 aussi dans « Work »
pub fn seed_server(store: &FakeMailStore, user: &str) {
    for (_, name, _) in BOXES.iter().filter(|(_, name, _)| *name != "INBOX") {
        store.create_mailbox(user, name, None);
    }
    for id in ["msg-1", "msg-2", "msg-3"] {
        let raw = mail(&format!("Sujet {id}"), &format!("{id}@example.org"));
        store.deliver(user, "INBOX", &raw, &[]);
    }
    let third = mail("Sujet msg-3", "msg-3@example.org");
    store.deliver(user, "Work", &third, &[]);
}

/// Reflète le serveur dans le miroir du compte : boîtes avec leur UIDVALIDITY, messages non lus, apparitions
pub fn seed_mirror(db: &Database, account: &AccountId, store: &FakeMailStore, user: &str) {
    let owner = account.as_str();
    db.write(|tx| {
        for (id, name, role) in BOXES {
            tx.upsert_mailbox(&sample_mailbox(id, owner, name, role))?;
            let validity = store.uidvalidity(user, name).unwrap_or(1);
            let marker = SyncMarker {
                uid_validity: validity,
                uid_next: 10,
                highest_modseq: None,
            };
            tx.set_mailbox_marker(&MailboxId::from_raw(id), Some(marker))?;
        }
        Ok(())
    })
    .unwrap();
    seed_messages(db, owner);
}

/// msg-1 à msg-3 dans la réception (UID 1 à 3), msg-3 aussi dans « Work » (UID 1)
fn seed_messages(db: &Database, owner: &str) {
    db.write(|tx| {
        for (index, id) in ["msg-1", "msg-2", "msg-3"].into_iter().enumerate() {
            let date = Timestamp::from_millis(1_000 * (index as i64 + 1));
            tx.insert_message(&sample_message(id, owner), date)?;
            tx.add_appearance(&appearance(id, "box-in", index as u32 + 1))?;
        }
        tx.add_appearance(&appearance("msg-3", "box-work", 1))?;
        for (id, ..) in BOXES {
            tx.refresh_mailbox_counters(&MailboxId::from_raw(id))?;
        }
        Ok(())
    })
    .unwrap();
}
