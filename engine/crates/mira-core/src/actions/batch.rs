//! Soumission d'un lot d'actions en une seule transaction : tout est mis en file, ou rien.

use mira_vocabulary::{ActionId, ActionKind, EventKind, MiraError, ProviderQuirks};

use super::errors::input::invalid_parameter;
use super::model::{QueueContext, SubmitRequest, UNDO_DELAY_MILLIS};
use super::queue::queue_action;

/// Une action du lot, avec les particularités du service de son compte.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchItem {
    pub quirks: ProviderQuirks,
    pub request: SubmitRequest,
}

/// Met toutes les actions du lot en file dans une transaction ; une seule cible invalide annule tout le lot
pub fn submit_batch(
    ctx: &QueueContext<'_>,
    items: &[BatchItem],
) -> Result<Vec<ActionId>, MiraError> {
    if items
        .iter()
        .any(|item| matches!(item.request.kind, ActionKind::Send { .. }))
    {
        return Err(invalid_parameter(
            "un envoi se programme avec schedule_send",
        ));
    }
    let not_before = ctx.clock.now().plus_millis(UNDO_DELAY_MILLIS);
    let queued = ctx.db.write(|tx| {
        items
            .iter()
            .map(|item| queue_action(tx, ctx, &item.quirks, &item.request, not_before))
            .collect::<Result<Vec<_>, _>>()
    })?;
    for entry in queued.iter().filter(|entry| entry.is_new) {
        ctx.events.emit(EventKind::ActionQueued {
            account: entry.action.account.clone(),
            action: entry.action.id.clone(),
        })?;
    }
    Ok(queued.into_iter().map(|entry| entry.action.id).collect())
}
