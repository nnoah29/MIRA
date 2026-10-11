//! Soumission d'une action : réponse immédiate, sans réseau (invariant 28).

use mira_vocabulary::{ActionId, ActionKind, EventKind, MiraError, ProviderQuirks};

use super::errors::input::invalid_parameter;
use super::model::{QueueContext, SubmitRequest, UNDO_DELAY_MILLIS};
use super::queue::queue_action;

/// Met l'action en file avec son effet local et rend son identifiant ; la même clé rend la même action (invariant 18)
pub fn submit(
    ctx: &QueueContext<'_>,
    quirks: &ProviderQuirks,
    request: SubmitRequest,
) -> Result<ActionId, MiraError> {
    if matches!(request.kind, ActionKind::Send { .. }) {
        return Err(invalid_parameter(
            "un envoi se programme avec schedule_send",
        ));
    }
    let not_before = ctx.clock.now().plus_millis(UNDO_DELAY_MILLIS);
    let queued = ctx
        .db
        .write(|tx| queue_action(tx, ctx, quirks, &request, not_before))?;
    if queued.is_new {
        ctx.events.emit(EventKind::ActionQueued {
            account: queued.action.account.clone(),
            action: queued.action.id.clone(),
        })?;
    }
    Ok(queued.action.id)
}
