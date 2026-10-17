//! Relevés du miroir : lignes (boîte, UID, objet, drapeaux), identifiants par objet, boîtes par rôle.

use std::collections::HashMap;

use mira_core::storage::Database;
use mira_vocabulary::{AccountId, Mailbox, MessageId};

/// Ligne comparable du miroir, indépendante des identifiants internes.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MirrorRow {
    pub mailbox: String,
    pub uid: u32,
    pub subject: String,
    pub seen: bool,
    pub flagged: bool,
}

/// Compte de test
pub fn account() -> AccountId {
    AccountId::from_raw("acc-sync")
}

/// Toutes les apparitions du miroir, triées
pub fn mirror_rows(db: &Database) -> Vec<MirrorRow> {
    let mut rows = Vec::new();
    db.read(|tx| {
        for mailbox in tx.list_mailboxes(&account())? {
            for found in tx.appearances_in_mailbox(&mailbox.id)? {
                let summary = tx.get_message(&found.message)?.unwrap();
                rows.push(MirrorRow {
                    mailbox: mailbox.name.clone(),
                    uid: found.uid,
                    subject: summary.subject,
                    seen: summary.flags.seen,
                    flagged: summary.flags.flagged,
                });
            }
        }
        Ok(())
    })
    .unwrap();
    rows.sort();
    rows
}

/// Identifiant interne de chaque message du miroir, par objet
pub fn ids_by_subject(db: &Database) -> HashMap<String, MessageId> {
    let mut ids = HashMap::new();
    db.read(|tx| {
        for mailbox in tx.list_mailboxes(&account())? {
            for found in tx.appearances_in_mailbox(&mailbox.id)? {
                let summary = tx.get_message(&found.message)?.unwrap();
                ids.insert(summary.subject, found.message);
            }
        }
        Ok(())
    })
    .unwrap();
    ids
}

/// Nombre de lignes de la table des messages du compte (apparitions ou non)
pub fn stored_messages(db: &Database) -> usize {
    let total = db.read(|tx| tx.table_row_count("messages")).unwrap();
    usize::try_from(total).unwrap()
}

/// Boîte du miroir portant ce nom
pub fn mailbox_named(db: &Database, name: &str) -> Mailbox {
    db.read(|tx| tx.find_mailbox_by_name(&account(), name))
        .unwrap()
        .unwrap()
}
