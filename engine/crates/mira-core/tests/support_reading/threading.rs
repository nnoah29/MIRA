//! Messages à en-têtes de fil et rattachement dans la base de test.

use std::collections::BTreeMap;

use mira_core::storage::Database;
use mira_core::threads::assign_thread;
use mira_vocabulary::{MessageId, MessageSummary, ThreadId, Timestamp};

use crate::support_mirror::fixtures::sample_message;

/// Message du compte acc-1 avec ses en-têtes de fil, sans aucun autre en-tête de réponse
pub fn threaded(
    id: &str,
    subject: &str,
    own: Option<&str>,
    in_reply_to: Option<&str>,
    references: &[&str],
) -> MessageSummary {
    let mut message = sample_message(id, "acc-1");
    message.subject = subject.to_string();
    message.message_id_header = own.map(str::to_string);
    message.in_reply_to = in_reply_to.map(str::to_string);
    message.references = references.iter().map(|value| value.to_string()).collect();
    message
}

/// Insère le message à la date donnée puis le rattache à son fil
pub fn store_and_assign(
    db: &Database,
    message: &MessageSummary,
    millis: i64,
    hint: Option<&str>,
) -> ThreadId {
    db.write(|tx| {
        tx.insert_message(message, Timestamp::from_millis(millis))?;
        let candidate = ThreadId::from_raw(format!("t-{}", message.id));
        assign_thread(tx, &candidate, message, hint)
    })
    .unwrap()
}

/// Groupes de messages par fil, sans l'identifiant du fil, triés pour comparaison
pub fn partition(db: &Database, account: &str) -> Vec<Vec<String>> {
    let rows = db
        .read(|tx| {
            let batch = tx.messages_for_threading(
                &mira_vocabulary::AccountId::from_raw(account),
                None,
                500,
            )?;
            Ok(batch)
        })
        .unwrap();
    let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (_, _, message) in rows {
        let key = message.thread.map(|t| t.to_string()).unwrap_or_default();
        groups.entry(key).or_default().push(message.id.to_string());
    }
    let mut result: Vec<Vec<String>> = groups.into_values().collect();
    result.sort();
    result
}

/// Fil d'un message, tel que stocké
pub fn thread_of(db: &Database, id: &str) -> Option<ThreadId> {
    db.read(|tx| tx.get_message(&MessageId::from_raw(id)))
        .unwrap()
        .unwrap()
        .thread
}
