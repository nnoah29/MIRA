//! Copie du message envoyé dans la boîte « Envoyés », seulement si le serveur ne la fait pas lui-même (10.8).

use mira_vocabulary::{EventKind, MailboxRole, MiraError, PendingAction};

use super::role_mailbox::role_mailbox_name;
use crate::actions::errors::input::role_mailbox_missing;
use crate::actions::replay::ReplayContext;
use crate::imap::{ImapClient, ImapFlag};
use crate::smtp::compose::BuiltMessage;

/// Dépose le message lu dans « Envoyés » (APPEND) puis annonce `SentCopyStored` ; sans effet si le serveur copie
pub(crate) async fn store_sent_copy(
    ctx: &ReplayContext<'_>,
    client: &mut ImapClient,
    action: &PendingAction,
    built: &BuiltMessage,
) -> Result<(), MiraError> {
    if ctx.quirks.server_saves_sent_copy {
        return Ok(());
    }
    let Some(mailbox) = role_mailbox_name(ctx, MailboxRole::Sent)? else {
        return Err(role_mailbox_missing("Sent"));
    };
    client
        .append(&mailbox, &[ImapFlag::Seen], &built.raw)
        .await?;
    ctx.queue.events.emit(EventKind::SentCopyStored {
        account: action.account.clone(),
        action: action.id.clone(),
    })?;
    Ok(())
}
