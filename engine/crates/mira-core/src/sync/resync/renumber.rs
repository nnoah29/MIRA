//! Resynchronisation d'une boîte renumérotée par le serveur (10.5, S13).

use mira_vocabulary::{EventKind, Mailbox, MiraError};

use super::purge::purge_unconfirmed;
use crate::imap::SelectedMailbox;
use crate::storage::Tx;
use crate::sync::session::SyncSession;
use crate::sync::strategy::{Announce, ingest_headers};

/// Marque les apparitions « à revalider », sauf si une passe interrompue l'a déjà fait.
fn prepare_revalidation(tx: &Tx<'_>, mailbox: &Mailbox) -> Result<(), MiraError> {
    let known = tx.appearances_in_mailbox(&mailbox.id)?;
    let interrupted = known.iter().any(|a| a.needs_revalidation && a.uid != 0);
    if interrupted {
        return Ok(());
    }
    tx.mark_mailbox_for_revalidation(&mailbox.id)?;
    tx.forget_uid_ids_for_mailbox(&mailbox.id)
}

/// Relit la plage, reconnaît les messages (identifiant stable, puis recoupement), purge le reste.
pub async fn resync_mailbox(
    session: &mut SyncSession,
    mailbox: &Mailbox,
    selected: &SelectedMailbox,
) -> Result<(), MiraError> {
    session
        .ctx
        .db
        .write(|tx| prepare_revalidation(tx, mailbox))?;
    let since = session
        .ctx
        .window_start(session.ctx.account.sync.window_days);
    let uids = session.conn.client.uids_since(since).await?;
    let validity = selected.uid_validity;
    ingest_headers(session, mailbox, validity, &uids, Announce::Silent).await?;
    session.ctx.db.write(|tx| purge_unconfirmed(tx, mailbox))?;
    session.ctx.emit(EventKind::MailboxResynchronized {
        account: session.ctx.account.id.clone(),
        mailbox: mailbox.id.clone(),
    });
    Ok(())
}
