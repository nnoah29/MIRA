//! « Remonter plus loin » (D-21, S12) : télécharge les en-têtes de la période précédente, annulable.

use std::collections::HashSet;

use mira_vocabulary::{EventKind, Mailbox, MailboxId, MiraError};

use crate::operations::OperationHandle;
use crate::storage::repositories::mailboxes::mailbox_not_found;
use crate::sync::download::count;
use crate::sync::session::SyncSession;
use crate::sync::strategy::{Announce, HEADER_BATCH, ingest_batch, newest_first};

/// Issue d'une extension de plage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtendOutcome {
    Finished,
    Cancelled,
}

/// UID du serveur plus récents que la nouvelle limite et encore inconnus du miroir.
async fn missing_uids(
    session: &mut SyncSession,
    mailbox: &Mailbox,
    days: u32,
) -> Result<Vec<u32>, MiraError> {
    let since = session.ctx.window_start(days);
    let listed = session.conn.client.uids_since(since).await?;
    let known = session.ctx.db.read(|tx| tx.uids_in_mailbox(&mailbox.id))?;
    let known: HashSet<u32> = known.into_iter().map(|(uid, _)| uid).collect();
    Ok(listed
        .into_iter()
        .filter(|uid| !known.contains(uid))
        .collect())
}

/// Élargit la plage du compte, seulement quand l'extension a réussi jusqu'au bout.
fn widen_window(session: &SyncSession, days: u32) -> Result<(), MiraError> {
    session.ctx.db.write(|tx| {
        let Some(mut account) = tx.get_account(&session.ctx.account.id)? else {
            return Ok(());
        };
        account.sync.window_days = days;
        tx.update_account(&account)
    })
}

/// Télécharge les lots d'en-têtes manquants en vérifiant l'annulation entre deux lots.
async fn download_missing(
    session: &mut SyncSession,
    handle: &OperationHandle,
    mailbox: &Mailbox,
    validity: u32,
    uids: &[u32],
) -> Result<ExtendOutcome, MiraError> {
    let ordered = newest_first(uids);
    let total = count(ordered.len());
    let mut done = 0;
    handle.progress(0, total);
    for batch in ordered.chunks(HEADER_BATCH) {
        if handle.is_cancelled() {
            return Ok(ExtendOutcome::Cancelled);
        }
        ingest_batch(session, mailbox, validity, batch, Announce::Silent).await?;
        done += count(batch.len());
        handle.progress(done, total);
    }
    Ok(ExtendOutcome::Finished)
}

/// Recalcule les compteurs de la boîte après l'ajout d'anciens messages.
fn refresh_counters(session: &SyncSession, mailbox: &MailboxId) -> Result<(), MiraError> {
    let fresh = session.ctx.db.write(|tx| {
        tx.refresh_mailbox_counters(mailbox)?;
        tx.get_mailbox(mailbox)
    })?;
    if let Some(box_now) = fresh {
        session.ctx.emit(EventKind::MailboxCountersUpdated {
            account: session.ctx.account.id.clone(),
            mailbox: box_now.id,
            total: box_now.total,
            unread: box_now.unread,
        });
    }
    Ok(())
}

/// Remonte la plage de `extra_days` jours pour une boîte, avec progression et annulation.
pub async fn extend_window(
    session: &mut SyncSession,
    handle: &OperationHandle,
    mailbox: &MailboxId,
    extra_days: u32,
) -> Result<ExtendOutcome, MiraError> {
    let found = session.ctx.db.read(|tx| tx.get_mailbox(mailbox))?;
    let mailbox = found.ok_or_else(|| mailbox_not_found(mailbox))?;
    let account = session
        .ctx
        .db
        .read(|tx| tx.get_account(&session.ctx.account.id))?;
    let current = account.map_or(session.ctx.account.sync.window_days, |a| a.sync.window_days);
    let days = current.saturating_add(extra_days);
    let selected = session.conn.client.select(&mailbox.name, None).await?;
    let uids = missing_uids(session, &mailbox, days).await?;
    let validity = selected.uid_validity;
    let outcome = download_missing(session, handle, &mailbox, validity, &uids).await?;
    refresh_counters(session, &mailbox.id)?;
    if outcome == ExtendOutcome::Finished {
        widen_window(session, days)?;
    }
    Ok(outcome)
}
