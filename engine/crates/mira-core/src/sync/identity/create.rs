//! Création d'un message du miroir à partir de ses en-têtes : ligne, apparition, fil, index.

use mira_vocabulary::{MessageId, MiraError, ThreadId};

use super::flags::is_deleted;
use super::incoming::Incoming;
use super::placement::place_appearance;
use super::remember::remember;
use super::summary::{address_text, build_summary, sort_date};
use crate::storage::Tx;
use crate::sync::context::SyncContext;
use crate::threads::assign_thread;

/// Crée le message, sa première apparition, ses correspondances serveur, son fil et son index.
pub fn create_message(
    tx: &Tx<'_>,
    ctx: &SyncContext,
    incoming: &Incoming,
) -> Result<MessageId, MiraError> {
    let id = MessageId::from_raw(ctx.new_id());
    let summary = build_summary(ctx, &id, incoming);
    tx.insert_message(&summary, sort_date(ctx, &summary))?;
    let deleted = is_deleted(&incoming.fetched.flags);
    place_appearance(tx, &id, &incoming.mailbox, incoming.uid(), deleted)?;
    remember(tx, ctx.account_id(), &id, incoming)?;
    let candidate = ThreadId::from_raw(ctx.new_id());
    let hint = incoming.translation.thread_hint.as_deref();
    assign_thread(tx, &candidate, &summary, hint)?;
    tx.index_message(&id, &summary.subject, &address_text(&summary), "")?;
    Ok(id)
}
