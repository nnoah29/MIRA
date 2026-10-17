//! Début et fin d'une tentative : « en cours » est écrit AVANT le serveur (inv. 17), « réussi » APRÈS.

use mira_vocabulary::{ActionKind, ActionState, EventKind, MiraError, PendingAction};

use super::context::ReplayContext;
use super::report::StepOutcome;
use crate::send::prepare_send_attempt;

/// Passe l'action « en attente » à « en cours » ; rend faux si elle a été annulée entre-temps
pub(super) fn begin_attempt(
    ctx: &ReplayContext<'_>,
    action: &PendingAction,
) -> Result<bool, MiraError> {
    ctx.queue.db.write(|tx| {
        if !tx.begin_action_attempt(&action.id)? {
            return Ok(false);
        }
        if let ActionKind::Send { draft } = &action.kind {
            prepare_send_attempt(tx, ctx.queue.ids, &action.id, draft)?;
        }
        Ok(true)
    })
}

/// Marque l'action réussie et publie l'événement qui l'annonce (les envois ont les leurs)
pub(super) fn complete_attempt(
    ctx: &ReplayContext<'_>,
    action: &PendingAction,
) -> Result<StepOutcome, MiraError> {
    ctx.queue
        .db
        .write(|tx| tx.set_action_state(&action.id, ActionState::Succeeded))?;
    if !matches!(action.kind, ActionKind::Send { .. }) {
        ctx.queue.events.emit(EventKind::ActionApplied {
            account: action.account.clone(),
            action: action.id.clone(),
        })?;
    }
    Ok(StepOutcome::Succeeded)
}
