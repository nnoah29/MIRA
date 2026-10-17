//! Décision de l'utilisateur sur un envoi incertain : renvoyer ou abandonner (décision 9, inv. 26).

use mira_vocabulary::{
    ActionId, ActionKind, ActionState, DraftState, EventKind, MiraError, PendingAction, Timestamp,
};

use super::message_id::clear_message_id;
use crate::actions::errors::incompatible::action_not_retryable;
use crate::actions::errors::not_found::action_not_found;
use crate::actions::{QueueContext, revert_local_effect};
use crate::drafts::state::set_draft_state;
use crate::storage::Tx;

/// Renvoie (nouvelle tentative, nouveau `Message-ID`) ou abandonne (le brouillon revient en édition) un envoi incertain
pub fn resolve_uncertain_send(
    ctx: &QueueContext<'_>,
    id: &ActionId,
    resend: bool,
) -> Result<(), MiraError> {
    let now = ctx.clock.now();
    let action = ctx.db.write(|tx| {
        let action = require_uncertain_send(tx.get_action(id)?, id)?;
        if resend {
            restart_send(tx, &action, now)?;
        } else {
            revert_local_effect(tx, &action)?;
            tx.set_action_state(id, ActionState::Cancelled)?;
        }
        Ok(action)
    })?;
    ctx.events.emit(decision_event(&action, resend, now))?;
    Ok(())
}

/// Événement de la décision : envoi reprogrammé, ou action abandonnée
fn decision_event(action: &PendingAction, resend: bool, now: Timestamp) -> EventKind {
    if resend {
        return EventKind::SendScheduled {
            account: action.account.clone(),
            action: action.id.clone(),
            at: now,
        };
    }
    EventKind::ActionCancelled {
        account: action.account.clone(),
        action: action.id.clone(),
    }
}

/// Exige un envoi à l'état incertain
fn require_uncertain_send(
    found: Option<PendingAction>,
    id: &ActionId,
) -> Result<PendingAction, MiraError> {
    let action = found.ok_or_else(|| action_not_found(id))?;
    let is_send = matches!(action.kind, ActionKind::Send { .. });
    if action.state == ActionState::Uncertain && is_send {
        return Ok(action);
    }
    Err(action_not_retryable(id))
}

/// Remet l'envoi en file, immédiatement rejouable, avec un `Message-ID` neuf
fn restart_send(tx: &Tx<'_>, action: &PendingAction, now: Timestamp) -> Result<(), MiraError> {
    if let ActionKind::Send { draft } = &action.kind {
        set_draft_state(tx, draft, DraftState::Scheduled { at: now })?;
    }
    clear_message_id(tx, &action.id)?;
    tx.requeue_action(&action.id, now)
}
