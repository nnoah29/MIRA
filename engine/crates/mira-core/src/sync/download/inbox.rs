//! Synchronisation de la seule réception, déclenchée par IDLE.

use mira_vocabulary::{MailboxRole, MiraError};

use super::account::{run_pass, sync_account};
use super::prefetch::prefetch_bodies;
use crate::sync::session::SyncSession;
use crate::sync::strategy::sync_mailbox;

/// Synchronise la réception (et ses corps récents) ; sans réception connue, tout le compte.
pub async fn sync_inbox(session: &mut SyncSession) -> Result<(), MiraError> {
    let account = session.ctx.account.id.clone();
    let found = session
        .ctx
        .db
        .read(|tx| tx.find_mailbox_by_role(&account, MailboxRole::Inbox))?;
    let Some(inbox) = found else {
        return sync_account(session).await;
    };
    run_pass(session, async |pass| {
        sync_mailbox(pass, &inbox).await?;
        prefetch_bodies(pass, &inbox).await
    })
    .await
}
