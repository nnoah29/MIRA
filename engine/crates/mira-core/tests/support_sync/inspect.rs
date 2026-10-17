//! Lectures du miroir pour les assertions : drapeaux, objets, état du compte, plage de synchronisation.

use mira_core::storage::Database;
use mira_vocabulary::{AccountState, Flags, MessageId, MessageSummary, Page, PageRequest};

use super::mirror::{account, mailbox_named, mirror_rows};

/// Nombre de jours de la plage de synchronisation enregistrée pour le compte
pub fn window_days(db: &Database) -> u32 {
    let stored = db.read(|tx| tx.get_account(&account())).unwrap();
    stored.unwrap().sync.window_days
}

/// État de connexion enregistré pour le compte
pub fn account_state(db: &Database) -> AccountState {
    let stored = db.read(|tx| tx.get_account(&account())).unwrap();
    stored.unwrap().state
}

/// Première page de la réception, du plus récent au plus ancien
pub fn inbox_page(db: &Database) -> Page<MessageSummary> {
    let inbox = mailbox_named(db, "INBOX");
    db.read(|tx| tx.list_messages_in_mailbox(&inbox.id, &PageRequest::default()))
        .unwrap()
}

/// Drapeaux d'un message
pub fn flags_of(db: &Database, id: &MessageId) -> Flags {
    db.read(|tx| tx.get_message(id)).unwrap().unwrap().flags
}

/// Objets des messages du miroir, dans l'ordre des lignes triées
pub fn subjects(db: &Database) -> Vec<String> {
    mirror_rows(db).into_iter().map(|row| row.subject).collect()
}

/// Change le nombre de corps préchargés du compte de test
pub fn set_prefetch(db: &Database, bodies: u32) {
    db.write(|tx| {
        let mut stored = tx.get_account(&account())?.unwrap();
        stored.sync.prefetch_bodies = bodies;
        tx.update_account(&stored)
    })
    .unwrap();
}
