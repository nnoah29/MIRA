//! Fils ajoutés directement au miroir du compte de la façade.

use mira_core::storage::Database;
use mira_core::threads::assign_thread;
use mira_vocabulary::{AccountId, MessageId, ThreadId, Timestamp};

use crate::support_mirror::fixtures::sample_message;

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
