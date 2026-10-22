//! Fils ajoutés directement au miroir du compte de la façade.

use mira_core::storage::Database;
use mira_core::threads::assign_thread;
use mira_vocabulary::{AccountId, MessageId, ThreadId, Timestamp};

use crate::support_mirror::fixtures::{appearance, sample_message};

/// Ajoute un message isolé (son propre fil) au compte et rend son fil et son identifiant
pub fn seed_thread(db: &Database, account: &AccountId, tag: &str) -> (ThreadId, MessageId) {
    let mut message = sample_message(&format!("msg-{tag}"), account.as_str());
    message.message_id_header = Some(format!("<{tag}@test>"));
    message.references.clear();
    message.subject = format!("Sujet {tag}");
    let thread = db
        .write(|tx| {
            tx.insert_message(&message, Timestamp::from_millis(1_000))?;
            let candidate = ThreadId::from_raw(format!("t-{tag}"));
            assign_thread(tx, &candidate, &message, None)
        })
        .unwrap();
    (thread, message.id)
}

/// Comme `seed_thread`, avec une apparition du message dans la réception (visible de l'aperçu des règles)
pub fn seed_inbox_thread(
    db: &Database,
    account: &AccountId,
    tag: &str,
    uid: u32,
) -> (ThreadId, MessageId) {
    let (thread, id) = seed_thread(db, account, tag);
    db.write(|tx| tx.add_appearance(&appearance(id.as_str(), "box-in", uid)))
        .unwrap();
    (thread, id)
}
