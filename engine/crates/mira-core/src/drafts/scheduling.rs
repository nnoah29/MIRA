//! Envoi planifié d'un brouillon (action `Send`) et annulation de cet envoi.

use mira_vocabulary::{
    ActionId, ActionKind, ActionState, Draft, DraftId, DraftState, EventKind, IdempotencyKey,
    MiraError, ProviderQuirks, Timestamp,
};

use super::state::set_draft_state;
use crate::actions::Queued;
use crate::actions::errors::incompatible::{draft_not_editable, no_send_to_cancel};
use crate::actions::errors::input::missing_parameter;
use crate::actions::errors::not_found::draft_not_found;
use crate::actions::{QueueContext, SEND_UNDO_DELAY_MILLIS, SubmitRequest, cancel, queue_action};
use crate::storage::Tx;

/// Met l'envoi du brouillon en file pour `at` (sinon dans 10 s, délai d'annulation) ; la même clé rend la même action
pub fn schedule_send(
    ctx: &QueueContext<'_>,
    draft: &DraftId,
    at: Option<Timestamp>,
    key: IdempotencyKey,
) -> Result<ActionId, MiraError> {
    let now = ctx.clock.now();
    let when = at.map_or(now.plus_millis(SEND_UNDO_DELAY_MILLIS), |wanted| {
        wanted.max(now)
    });
    let queued = ctx.db.write(|tx| queue_send(tx, ctx, draft, key, when))?;
    if queued.is_new {
        ctx.events.emit(EventKind::SendScheduled {
            account: queued.action.account.clone(),
            action: queued.action.id.clone(),
            at: when,
        })?;
    }
    Ok(queued.action.id)
}

/// Annule l'envoi tant qu'il est en attente : le brouillon revient en édition (`ActionCancelled` est émis)
pub fn cancel_send(ctx: &QueueContext<'_>, draft: &DraftId) -> Result<(), MiraError> {
    let action = ctx.db.read(|tx| find_open_send(tx, draft))?;
    cancel(ctx, &action)
}

/// Met l'envoi en file et planifie le brouillon ; une clé déjà connue ne change rien (invariant 18)
fn queue_send(
    tx: &Tx<'_>,
    ctx: &QueueContext<'_>,
    draft: &DraftId,
    key: IdempotencyKey,
    when: Timestamp,
) -> Result<Queued, MiraError> {
    let found = tx.get_draft(draft)?.ok_or_else(|| draft_not_found(draft))?;
    let request = send_request(&found, key);
    let known = tx
        .find_action_by_idempotency_key(&found.account, &request.key)?
        .is_some();
    if !known {
        require_sendable(&found)?;
    }
    let queued = queue_action(tx, ctx, &ProviderQuirks::default(), &request, when)?;
    if queued.is_new {
        set_draft_state(tx, draft, DraftState::Scheduled { at: when })?;
    }
    Ok(queued)
}

/// Demande de mise en file de l'envoi du brouillon
fn send_request(draft: &Draft, key: IdempotencyKey) -> SubmitRequest {
    SubmitRequest {
        account: draft.account.clone(),
        kind: ActionKind::Send {
            draft: draft.id.clone(),
        },
        targets: Vec::new(),
        key,
    }
}

/// Exige un brouillon en édition qui a au moins un destinataire
fn require_sendable(draft: &Draft) -> Result<(), MiraError> {
    if draft.state != DraftState::Editing {
        return Err(draft_not_editable());
    }
    if draft.to.is_empty() && draft.cc.is_empty() && draft.bcc.is_empty() {
        return Err(missing_parameter("le brouillon n'a aucun destinataire"));
    }
    Ok(())
}

/// Retrouve l'envoi encore ouvert (en attente ou en cours) du brouillon, le plus récent d'abord
fn find_open_send(tx: &Tx<'_>, draft: &DraftId) -> Result<ActionId, MiraError> {
    let found = tx.get_draft(draft)?.ok_or_else(|| draft_not_found(draft))?;
    let open = [ActionState::Pending, ActionState::InProgress];
    tx.list_actions(&found.account, &open)?
        .into_iter()
        .rev()
        .find(|action| {
            action.kind
                == ActionKind::Send {
                    draft: draft.clone(),
                }
        })
        .map(|action| action.id)
        .ok_or_else(no_send_to_cancel)
}
