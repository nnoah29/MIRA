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

/// Recoupement de dernier recours : un seul candidat qui attend une revalidation ou qui est orphelin, sinon rien.
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
    let kinds = classify(tx, &candidates, &incoming.mailbox)?;
    Ok(single_match(&candidates, &kinds))
}

/// Ce qu'un candidat du recoupement est pour la boîte qui reçoit le message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Candidate {
    AwaitingRevalidation,
    Orphan,
    Live,
}

/// Classe chaque candidat : à revalider dans cette boîte, orphelin de la passe (aucune apparition), ou vivant ailleurs.
fn classify(
    tx: &Tx<'_>,
    candidates: &[MessageId],
    mailbox: &MailboxId,
) -> Result<Vec<Candidate>, MiraError> {
    let mut kinds = Vec::new();
    for candidate in candidates {
        let waiting = tx.get_appearance(candidate, mailbox)?;
        kinds.push(if waiting.is_some_and(|found| found.needs_revalidation) {
            Candidate::AwaitingRevalidation
        } else if tx.appearances_of_message(candidate)?.is_empty() {
            Candidate::Orphan
        } else {
            Candidate::Live
        });
    }
    Ok(kinds)
}

/// Le candidat à reconnaître : unique parmi ceux qui attendent ou sont orphelins ; un orphelin n'est repris qu'en l'absence de jumeau vivant.
fn single_match(candidates: &[MessageId], kinds: &[Candidate]) -> Option<MessageId> {
    let eligible: Vec<usize> = (0..kinds.len())
        .filter(|index| kinds[*index] != Candidate::Live)
        .collect();
    let [only] = eligible.as_slice() else {
        return None;
    };
    let twin_alive = kinds.contains(&Candidate::Live);
    if kinds[*only] == Candidate::Orphan && twin_alive {
        return None;
    }
    candidates.get(*only).cloned()
}
