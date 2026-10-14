//! Resynchronisation demandée par l'utilisateur, sans que le serveur ait changé de numéro de validité.

use mira_vocabulary::{MailboxId, MiraError};

use super::renumber::resync_mailbox;
use crate::storage::repositories::mailboxes::mailbox_not_found;
use crate::sync::session::SyncSession;
use crate::sync::strategy::finish_mailbox;

/// Relit toute la plage d'une boîte comme après une renumérotation, puis avance son repère.
pub async fn force_resync(session: &mut SyncSession, mailbox: &MailboxId) -> Result<(), MiraError> {
    let found = session.ctx.db.read(|tx| tx.get_mailbox(mailbox))?;
    let known = found.ok_or_else(|| mailbox_not_found(mailbox))?;
    let selected = session.conn.client.select(&known.name, None).await?;
    resync_mailbox(session, &known, &selected).await?;
    finish_mailbox(session, &known, &selected)
}
