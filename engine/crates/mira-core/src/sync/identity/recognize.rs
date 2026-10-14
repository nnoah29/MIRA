//! Reconnaissance d'un message du serveur : identifiant stable, UID, puis recoupement prudent (D-13, P7).

use mira_vocabulary::{AccountId, MailboxId, MessageId, MiraError};

use super::incoming::Incoming;
use crate::storage::Tx;
use crate::storage::repositories::server_ids::ServerIdKind;

/// Retrouve le message local correspondant, ou rien si c'est un nouveau message.
pub fn recognize(
    tx: &Tx<'_>,
    account: &AccountId,
    incoming: &Incoming,
) -> Result<Option<MessageId>, MiraError> {
    if let Some(found) = by_stable(tx, account, incoming)? {
        return Ok(Some(found));
    }
    let by_uid = tx.find_by_server_id(account, ServerIdKind::Uid, &incoming.uid_key())?;
    match by_uid {
        Some(found) => Ok(Some(found)),
        None => by_fingerprint(tx, account, incoming),
    }
}

/// Correspondance par l'identifiant stable annoncé par le serveur, s'il y en a un.
fn by_stable(
    tx: &Tx<'_>,
    account: &AccountId,
    incoming: &Incoming,
) -> Result<Option<MessageId>, MiraError> {
    match &incoming.translation.stable_id {
        Some(stable) => tx.find_by_server_id(account, ServerIdKind::Stable, stable),
        None => Ok(None),
    }
}

/// Recoupement de dernier recours : un seul candidat qui attend une revalidation, sinon rien.
fn by_fingerprint(
    tx: &Tx<'_>,
    account: &AccountId,
    incoming: &Incoming,
) -> Result<Option<MessageId>, MiraError> {
    let Some(header) = incoming.parsed.message_id.as_deref() else {
        return Ok(None);
    };
    let date = incoming.parsed.date;
    let candidates = tx.find_by_fingerprint(account, header, date, incoming.fetched.size)?;
    let mut waiting = awaiting_revalidation(tx, candidates, &incoming.mailbox)?;
    match (waiting.pop(), waiting.is_empty()) {
        (Some(only), true) => Ok(Some(only)),
        _ => Ok(None),
    }
}

/// Garde les candidats dont l'apparition dans cette boîte attend une revalidation.
fn awaiting_revalidation(
    tx: &Tx<'_>,
    candidates: Vec<MessageId>,
    mailbox: &MailboxId,
) -> Result<Vec<MessageId>, MiraError> {
    let mut waiting = Vec::new();
    for candidate in candidates {
        let appearance = tx.get_appearance(&candidate, mailbox)?;
        if appearance.is_some_and(|found| found.needs_revalidation) {
            waiting.push(candidate);
        }
    }
    Ok(waiting)
}
