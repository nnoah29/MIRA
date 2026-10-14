//! Envoi interrompu (décision 9, inv. 26) : le `Message-ID` est cherché dans Envoyés ; jamais de renvoi automatique.

use mira_vocabulary::error::codes::server;
use mira_vocabulary::{ActionState, EventKind, MailboxRole, MiraError, PendingAction};

use super::draft_copy::purge_draft_copy;
use super::execute::draft_of;
use super::message_id::stored_message_id;
use super::role_mailbox::role_mailbox_name;
use crate::actions::replay::{ReplayContext, StepOutcome};
use crate::imap::ImapClient;

/// Règle l'envoi dont la connexion s'est coupée : trouvé dans Envoyés = réussi, sinon incertain
pub(crate) async fn settle_interrupted_send(
    ctx: &ReplayContext<'_>,
    client: &mut ImapClient,
    action: &PendingAction,
) -> Result<StepOutcome, MiraError> {
    let found = find_in_sent(ctx, client, action).await.unwrap_or(false);
    conclude_with_cleanup(ctx, client, action, found).await
}

/// Règle l'envoi laissé « en cours » par un arrêt brutal ; si Envoyés est illisible, l'envoi reste « en cours »
pub(crate) async fn recover_interrupted_send(
    ctx: &ReplayContext<'_>,
    client: &mut ImapClient,
    action: &PendingAction,
) -> Result<StepOutcome, MiraError> {
    let found = find_in_sent(ctx, client, action).await?;
    conclude_with_cleanup(ctx, client, action, found).await
}

/// Écrit le verdict ; un envoi retrouvé fait aussi purger la copie serveur du brouillon, sans jamais échouer pour cela
async fn conclude_with_cleanup(
    ctx: &ReplayContext<'_>,
    client: &mut ImapClient,
    action: &PendingAction,
    found: bool,
) -> Result<StepOutcome, MiraError> {
    if found {
        let purged = purge_draft_copy(ctx, client, draft_of(action)?).await;
        if let Err(error) = purged {
            tracing::warn!(code = %error.code, "copie serveur du brouillon non purgée");
        }
    }
    conclude(ctx, action, found)
}

/// Écrit le verdict : réussi si le message est dans Envoyés, incertain sinon
fn conclude(
    ctx: &ReplayContext<'_>,
    action: &PendingAction,
    found: bool,
) -> Result<StepOutcome, MiraError> {
    if !found {
        ctx.queue
            .db
            .write(|tx| tx.set_action_state(&action.id, ActionState::Uncertain))?;
        ctx.queue.events.emit(EventKind::SendUncertain {
            account: action.account.clone(),
            action: action.id.clone(),
        })?;
        return Ok(StepOutcome::Uncertain);
    }
    let draft = draft_of(action)?;
    ctx.queue.db.write(|tx| {
        tx.delete_draft(draft)?;
        tx.set_action_state(&action.id, ActionState::Succeeded)
    })?;
    ctx.queue.events.emit(EventKind::SendDeparted {
        account: action.account.clone(),
        action: action.id.clone(),
    })?;
    Ok(StepOutcome::Succeeded)
}

/// Cherche le `Message-ID` de l'envoi dans la boîte Envoyés du serveur (SEARCH HEADER)
async fn find_in_sent(
    ctx: &ReplayContext<'_>,
    client: &mut ImapClient,
    action: &PendingAction,
) -> Result<bool, MiraError> {
    let stored = ctx.queue.db.read(|tx| stored_message_id(tx, &action.id))?;
    let (Some(message_id), Some(sent)) = (stored, role_mailbox_name(ctx, MailboxRole::Sent)?)
    else {
        return Ok(false);
    };
    match client.select(&sent, None).await {
        Ok(_) => Ok(!client.uids_with_message_id(&message_id).await?.is_empty()),
        Err(error) if error.code == server::MAILBOX_MISSING.0 => Ok(false),
        Err(error) => Err(error),
    }
}
