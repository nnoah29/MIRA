//! Sort d'une action dont la tentative a échoué : suspendue, en attente d'authentification, ou échec définitif.

use mira_vocabulary::error::codes::network;
use mira_vocabulary::{ActionKind, ActionState, EventKind, MiraError, PendingAction};

use super::classify::{FailureClass, classify};
use super::context::ReplayContext;
use super::park::{park_for_authentication, park_transient};
use super::report::StepOutcome;
use crate::imap::ImapClient;
use crate::send::settle_interrupted_send;

/// Applique les règles d'échec (10.7) : une connexion coupée en plein envoi reste incertaine, jamais rejouée
pub(super) async fn settle_failure(
    ctx: &ReplayContext<'_>,
    client: &mut ImapClient,
    action: &PendingAction,
    error: &MiraError,
) -> Result<StepOutcome, MiraError> {
    let is_send = matches!(action.kind, ActionKind::Send { .. });
    if is_send && error.code == network::CONNECTION_LOST.0 {
        return settle_interrupted_send(ctx, client, action).await;
    }
    match classify(error) {
        FailureClass::Transient => park_transient(ctx, action, error),
        FailureClass::Authentication => park_for_authentication(ctx, action, error),
        FailureClass::Definitive => fail_definitively(ctx, action, error),
    }
}

/// Échec définitif : effet local annulé (inv. 22), état « échoué », événement
fn fail_definitively(
    ctx: &ReplayContext<'_>,
    action: &PendingAction,
    error: &MiraError,
) -> Result<StepOutcome, MiraError> {
    let now = ctx.queue.clock.now();
    ctx.queue.db.write(|tx| {
        crate::actions::revert_local_effect(tx, action)?;
        tx.record_action_failure(&action.id, &error.to_string(), now)?;
        tx.set_action_state(&action.id, ActionState::Failed)
    })?;
    ctx.queue.events.emit(failure_event(action, error))?;
    Ok(StepOutcome::Failed)
}

/// Événement d'échec : propre à l'envoi pour un envoi, générique sinon
fn failure_event(action: &PendingAction, error: &MiraError) -> EventKind {
    match action.kind {
        ActionKind::Send { .. } => EventKind::SendFailed {
            account: action.account.clone(),
            action: action.id.clone(),
            code: error.code.clone(),
        },
        _ => EventKind::ActionFailed {
            account: action.account.clone(),
            action: action.id.clone(),
            code: error.code.clone(),
            reason: error.message.clone(),
        },
    }
}
