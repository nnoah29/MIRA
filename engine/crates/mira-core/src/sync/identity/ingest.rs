//! Intégration d'un message du serveur au miroir : reconnu (serveur gagnant sur les drapeaux) ou créé.

use mira_vocabulary::{AccountId, Flags, MessageId, MiraError};

use super::create::create_message;
use super::flags::{is_deleted, server_flags};
use super::incoming::Incoming;
use super::placement::place_appearance;
use super::recognize::recognize;
use super::remember::remember;
use crate::storage::Tx;
use crate::sync::context::SyncContext;

/// Ce qu'a produit l'intégration d'un message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ingested {
    Created(MessageId),
    Updated(MessageId),
    Unchanged(MessageId),
}

/// Reconnaît le message ou le crée ; un message reconnu reçoit les drapeaux du serveur.
pub fn ingest_header(
    tx: &Tx<'_>,
    ctx: &SyncContext,
    incoming: &Incoming,
) -> Result<Ingested, MiraError> {
    match recognize(tx, ctx.account_id(), incoming)? {
        None => create_message(tx, ctx, incoming).map(Ingested::Created),
        Some(known) => refresh_known(tx, ctx.account_id(), known, incoming),
    }
}

/// Met à jour apparition, correspondances et drapeaux d'un message déjà connu.
fn refresh_known(
    tx: &Tx<'_>,
    account: &AccountId,
    known: MessageId,
    incoming: &Incoming,
) -> Result<Ingested, MiraError> {
    let deleted = is_deleted(&incoming.fetched.flags);
    let moved = place_appearance(tx, &known, &incoming.mailbox, incoming.uid(), deleted)?;
    remember(tx, account, &known, incoming)?;
    let flags = server_flags(&incoming.fetched.flags);
    let changed = apply_server_flags(tx, &known, flags)?;
    Ok(if moved || changed {
        Ingested::Updated(known)
    } else {
        Ingested::Unchanged(known)
    })
}

/// Écrase les drapeaux du miroir par ceux du serveur ; rend `true` s'ils différaient.
pub fn apply_server_flags(
    tx: &Tx<'_>,
    message: &MessageId,
    flags: Flags,
) -> Result<bool, MiraError> {
    let current = tx.get_message(message)?.map(|summary| summary.flags);
    if current == Some(flags) {
        return Ok(false);
    }
    tx.update_message_flags(message, flags)?;
    Ok(true)
}
