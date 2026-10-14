//! Copie du brouillon dans la boîte Brouillons du serveur (8.2) : l'ancienne copie est remplacée.

use mira_vocabulary::error::codes::invalid_input;
use mira_vocabulary::{DraftId, MailboxRole, MiraError};

use super::prepare::compose_draft;
use super::role_mailbox::role_mailbox_name;
use crate::actions::errors::input::role_mailbox_missing;
use crate::actions::replay::{ReplayContext, effective, purge_selected};
use crate::imap::{ImapClient, ImapFlag};
use crate::smtp::compose::BuiltMessage;

/// Enregistre le brouillon sur le serveur ; sans destinataire il ne peut pas être composé : rien n'est copié
pub(crate) async fn save_draft_copy(
    ctx: &ReplayContext<'_>,
    client: &mut ImapClient,
    draft: &DraftId,
) -> Result<(), MiraError> {
    let Some(mailbox) = role_mailbox_name(ctx, MailboxRole::Drafts)? else {
        return Err(role_mailbox_missing("Drafts"));
    };
    let message_id = format!("<draft-{draft}@mira.local>");
    match compose_draft(&ctx.queue, draft, &message_id) {
        Ok(built) => replace_copy(ctx, client, &mailbox, &built).await,
        Err(error) if error.code == invalid_input::MISSING_PARAMETER.0 => Ok(()),
        Err(error) => Err(error),
    }
}

/// Dépose la nouvelle copie puis supprime les précédentes (même `Message-ID`)
async fn replace_copy(
    ctx: &ReplayContext<'_>,
    client: &mut ImapClient,
    mailbox: &str,
    built: &BuiltMessage,
) -> Result<(), MiraError> {
    client.select(mailbox, None).await?;
    let previous = client.uids_with_message_id(&built.message_id).await?;
    let flags = [ImapFlag::Draft, ImapFlag::Seen];
    client.append(mailbox, &flags, &built.raw).await?;
    let capabilities = effective(client, ctx.quirks);
    purge_selected(client, &capabilities, &previous).await
}
