//! Rattachement d'un message à son fil, dans la transaction de l'appelant (synchro ou recalcul).

use mira_vocabulary::{MessageSummary, MiraError, ThreadId};

use super::key::thread_key;
use super::subject::strip_reply_prefixes;
use crate::storage::Tx;

/// Rattache un message déjà inséré à son fil et rend ce fil ; `candidate` sert d'identifiant si le fil est neuf
pub fn assign_thread(
    tx: &Tx<'_>,
    candidate: &ThreadId,
    summary: &MessageSummary,
    hint: Option<&str>,
) -> Result<ThreadId, MiraError> {
    let thread = match thread_of_parent(tx, summary)? {
        Some(known) => known,
        None => {
            let key = thread_key(summary, hint);
            tx.upsert_thread(&summary.account, &key, &thread_subject(summary), candidate)?
        }
    };
    tx.set_thread(&summary.id, Some(&thread))?;
    Ok(thread)
}

/// Fil du message connu désigné par `In-Reply-To`, même si la racine de ce fil diffère
fn thread_of_parent(tx: &Tx<'_>, summary: &MessageSummary) -> Result<Option<ThreadId>, MiraError> {
    let parent = summary.in_reply_to.as_deref().map(str::trim);
    match parent.filter(|value| !value.is_empty()) {
        Some(header) => tx.thread_of_known_header(&summary.account, header, &summary.id),
        None => Ok(None),
    }
}

/// Objet du fil : celui du message sans préfixes de réponse, ou tel quel s'il ne reste rien
fn thread_subject(summary: &MessageSummary) -> String {
    let stripped = strip_reply_prefixes(&summary.subject);
    if stripped.is_empty() {
        return summary.subject.clone();
    }
    stripped.to_string()
}
