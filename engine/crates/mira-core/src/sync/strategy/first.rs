//! Première synchronisation d'une boîte : les en-têtes de la plage, du plus récent au plus ancien.

use std::collections::HashSet;

use mira_vocabulary::{Mailbox, MiraError};

use super::flag_changes::remove_vanished;
use super::headers::{Announce, ingest_headers};
use crate::imap::SelectedMailbox;
use crate::sync::session::SyncSession;

/// Télécharge les en-têtes de la plage, sans annoncer de nouveaux messages (seulement la progression).
pub async fn first_sync(
    session: &mut SyncSession,
    mailbox: &Mailbox,
    selected: &SelectedMailbox,
) -> Result<(), MiraError> {
    let days = session.ctx.account.sync.window_days;
    let since = session.ctx.window_start(days);
    let uids = session.conn.client.uids_since(since).await?;
    let validity = selected.uid_validity;
    ingest_headers(session, mailbox, validity, &uids, Announce::Silent).await?;
    drop_unlisted(session, mailbox, &uids)
}

/// Retire les apparitions locales que le serveur ne liste plus (reprise après une passe interrompue).
fn drop_unlisted(
    session: &SyncSession,
    mailbox: &Mailbox,
    listed: &[u32],
) -> Result<(), MiraError> {
    let listed: HashSet<u32> = listed.iter().copied().collect();
    session.ctx.db.write(|tx| {
        let local = tx.uids_in_mailbox(&mailbox.id)?;
        let gone: Vec<u32> = local
            .into_iter()
            .map(|(uid, _)| uid)
            .filter(|uid| !listed.contains(uid))
            .collect();
        remove_vanished(tx, &mailbox.id, &gone).map(|_| ())
    })
}
