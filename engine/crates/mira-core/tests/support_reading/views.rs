//! Bases de test pour les vues : boîtes supplémentaires, messages à drapeaux, lecture des pages.

use mira_core::storage::Database;
use mira_vocabulary::{
    AddressEntry, Flags, MailboxRole, MessageId, MessageSummary, Page, Timestamp,
};

use crate::support_mirror::fixtures::{appearance, sample_mailbox, sample_message};
use crate::support_mirror::seeded::database_with_two_accounts;

/// Base avec acc-1 (box-1 boîte de réception, box-1b seconde réception, sent-1 envoyés, label-1 autre) et acc-2 (box-2)
pub fn database_with_many_boxes() -> Database {
    let db = database_with_two_accounts();
    db.write(|tx| {
        for (id, name, role) in [
            ("box-1b", "INBOX.bis", MailboxRole::Inbox),
            ("sent-1", "Sent", MailboxRole::Sent),
            ("label-1", "Label", MailboxRole::Other),
        ] {
            tx.upsert_mailbox(&sample_mailbox(id, "acc-1", name, role))?;
        }
        Ok(())
    })
    .unwrap();
    db
}

/// Insère un message, avec une apparition par boîte donnée (UID = date en secondes)
pub fn put_custom(db: &Database, message: &MessageSummary, millis: i64, boxes: &[&str]) {
    let uid = u32::try_from(millis / 1_000).unwrap();
    db.write(|tx| {
        tx.insert_message(message, Timestamp::from_millis(millis))?;
        for mailbox in boxes {
            tx.add_appearance(&appearance(message.id.as_str(), mailbox, uid))?;
        }
        Ok(())
    })
    .unwrap();
}

/// Insère un message ordinaire (non lu, sans drapeau) du compte donné
pub fn put_message(db: &Database, id: &str, account: &str, millis: i64, boxes: &[&str]) {
    put_custom(db, &sample_message(id, account), millis, boxes);
}

/// Identifiants des messages d'une page, dans l'ordre rendu
pub fn page_ids(page: &Page<MessageSummary>) -> Vec<String> {
    page.items.iter().map(|m| m.id.to_string()).collect()
}

/// Fixe les drapeaux lu et suivi d'un message
pub fn set_flags(db: &Database, id: &str, seen: bool, flagged: bool) {
    let flags = Flags {
        seen,
        flagged,
        ..Flags::default()
    };
    db.write(|tx| tx.update_message_flags(&MessageId::from_raw(id), flags))
        .unwrap();
}

/// Liste d'expéditeurs à une entrée, pour fabriquer des messages de provenance connue
pub fn sender(name: &str, address: &str) -> Vec<AddressEntry> {
    vec![AddressEntry {
        name: Some(name.to_string()),
        address: address.to_string(),
    }]
}
