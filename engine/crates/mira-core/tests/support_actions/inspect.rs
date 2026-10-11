//! Lectures de contrôle du miroir et de la file pour les assertions.

use mira_core::storage::Database;
use mira_vocabulary::{
    ActionId, ActionState, Mailbox, MailboxId, MessageId, MessageSummary, PendingAction,
};

use super::world::World;

/// Résumé d'un message du monde de test
pub fn message(db: &Database, id: &str) -> MessageSummary {
    db.read(|tx| tx.get_message(&MessageId::from_raw(id)))
        .unwrap()
        .unwrap()
}

/// Boîte du monde de test, compteurs inclus
pub fn mailbox(db: &Database, id: &str) -> Mailbox {
    db.read(|tx| tx.get_mailbox(&MailboxId::from_raw(id)))
        .unwrap()
        .unwrap()
}

/// Identifiants des boîtes où le message est visible, triés
pub fn boxes_of(db: &Database, id: &str) -> Vec<String> {
    message(db, id)
        .mailboxes
        .iter()
        .map(|mailbox| mailbox.as_str().to_string())
        .collect()
}

/// Action relue en base
pub fn action(db: &Database, id: &ActionId) -> PendingAction {
    db.read(|tx| tx.get_action(id)).unwrap().unwrap()
}

/// Force l'état d'une action, comme le ferait le rejeu
pub fn force_state(world: &World, id: &ActionId, state: ActionState) {
    world.db.write(|tx| tx.set_action_state(id, state)).unwrap();
}

/// Nombre de lignes de la file d'actions
pub fn queue_len(db: &Database) -> i64 {
    db.read(|tx| tx.table_row_count("actions")).unwrap()
}
