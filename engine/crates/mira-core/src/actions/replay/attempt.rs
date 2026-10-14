//! Début et fin d'une tentative : « en cours » est écrit AVANT le serveur (inv. 17), « réussi » APRÈS.

use mira_vocabulary::{ActionKind, ActionState, EventKind, MiraError, PendingAction};

use super::context::ReplayContext;
use super::report::StepOutcome;
use crate::send::prepare_send_attempt;

/// Écrit l'état « en cours » dans la base ; pour un envoi, fixe aussi le `Message-ID` et fige le brouillon
pub(super) fn begin_attempt(
    ctx: &ReplayContext<'_>,
    action: &PendingAction,
) -> Result<(), MiraError> {
    ctx.queue.db.write(|tx| {
        if let ActionKind::Send { draft } = &action.kind {
            prepare_send_attempt(tx, ctx.queue.ids, &action.id, draft)?;
        }
        tx.set_action_state(&action.id, ActionState::InProgress)
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
