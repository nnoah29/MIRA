//! Exécution d'un envoi : MIME, SMTP, puis (sans jamais échouer) copie dans Envoyés et nettoyage du brouillon.

use mira_vocabulary::error::codes::internal;
use mira_vocabulary::{ActionKind, DraftId, ErrorCategory, EventKind, MiraError, PendingAction};

use super::deliver::deliver;
use super::draft_copy::purge_draft_copy;
use super::message_id::stored_message_id;
use super::prepare::compose_draft;
use super::sent_copy::store_sent_copy;
use crate::actions::replay::ReplayContext;
use crate::imap::ImapClient;
use crate::smtp::compose::BuiltMessage;

/// Envoie le brouillon de l'action ; une fois le message parti, plus aucune erreur ne remonte (pas de renvoi)
pub(crate) async fn execute_send(
    ctx: &ReplayContext<'_>,
    client: &mut ImapClient,
    action: &PendingAction,
) -> Result<(), MiraError> {
    let draft = draft_of(action)?;
    let stored = ctx.queue.db.read(|tx| stored_message_id(tx, &action.id))?;
    let message_id = stored.ok_or_else(|| bug("Message-ID de l'envoi non mémorisé"))?;
    let built = compose_draft(&ctx.queue, draft, &message_id)?;
    deliver(ctx, &built).await?;
    after_departure(ctx, client, action, &built).await;
    Ok(())
}

/// Brouillon que l'action envoie
pub(crate) fn draft_of(action: &PendingAction) -> Result<&DraftId, MiraError> {
    match &action.kind {
        ActionKind::Send { draft } => Ok(draft),
        _ => Err(bug("l'action n'est pas un envoi")),
    }
}

/// Erreur interne : situation que le moteur ne doit jamais rencontrer
fn bug(message: &str) -> MiraError {
    MiraError::new(ErrorCategory::Internal, internal::BUG, message)
}

/// Étapes qui suivent l'acceptation par le serveur SMTP : chacune est notée en cas d'échec mais n'arrête rien
async fn after_departure(
    ctx: &ReplayContext<'_>,
    client: &mut ImapClient,
    action: &PendingAction,
    built: &BuiltMessage,
) {
    let departed = EventKind::SendDeparted {
        account: action.account.clone(),
        action: action.id.clone(),
    };
    note_failure(ctx.queue.events.emit(departed).map(|_| ()));
    note_failure(store_sent_copy(ctx, client, action, built).await);
    if let Ok(draft) = draft_of(action) {
        note_failure(purge_draft_copy(ctx, client, draft).await);
        note_failure(ctx.queue.db.write(|tx| tx.delete_draft(draft)));
    }
}

/// Note dans le journal l'échec d'une étape secondaire, sans secret ni contenu de message
fn note_failure(result: Result<(), MiraError>) {
    if let Err(error) = result {
        tracing::warn!(code = %error.code, "étape secondaire de l'envoi non terminée");
    }
}
