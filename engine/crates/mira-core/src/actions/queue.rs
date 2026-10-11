//! Mise en file dans une transaction : enqueue idempotent, effet local, inverse et cibles serveur mémorisés.

use mira_vocabulary::{
    AccountId, ActionId, ActionState, MiraError, PendingAction, ProviderQuirks, Timestamp,
};

use super::effect::apply_all;
use super::errors::not_found::account_not_found;
use super::model::{QueueContext, SubmitRequest, UndoRecord};
use super::plan::steps_for;
use super::resolve::resolve_intent;
use super::targets::capture_server_targets;
use crate::storage::Tx;
use crate::storage::json_columns::to_json;
use crate::storage::repositories::actions::ActionPayload;

/// Action mise en file, et dit si cet appel l'a créée (faux : même clé déjà enregistrée, aucun second effet).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Queued {
    pub(crate) action: PendingAction,
    pub(crate) is_new: bool,
}

/// Met l'action en file puis applique son effet local, dans la transaction de l'appelant
pub(crate) fn queue_action(
    tx: &Tx<'_>,
    ctx: &QueueContext<'_>,
    quirks: &ProviderQuirks,
    request: &SubmitRequest,
    not_before: Timestamp,
) -> Result<Queued, MiraError> {
    require_account(tx, &request.account)?;
    let candidate = candidate_action(ctx, request, not_before);
    let stored = tx.enqueue_action(&candidate)?;
    if stored.id != candidate.id {
        return Ok(Queued {
            action: stored,
            is_new: false,
        });
    }
    apply_and_record(tx, ctx, quirks, request, &stored.id)?;
    Ok(Queued {
        action: stored,
        is_new: true,
    })
}

/// Résout l'effet, relève les cibles serveur, applique l'effet et mémorise cibles et inverse
fn apply_and_record(
    tx: &Tx<'_>,
    ctx: &QueueContext<'_>,
    quirks: &ProviderQuirks,
    request: &SubmitRequest,
    id: &ActionId,
) -> Result<(), MiraError> {
    let intent = resolve_intent(tx, request, quirks, ctx.ids)?;
    let server_targets =
        capture_server_targets(tx, &request.account, &request.kind, &request.targets)?;
    let inverse = apply_all(tx, &steps_for(tx, &intent, &request.targets)?)?;
    tx.set_action_payload(
        id,
        &ActionPayload {
            server_targets_json: to_json(&server_targets)?,
            undo_json: Some(to_json(&UndoRecord { intent, inverse })?),
        },
    )
}

/// Exige que le compte existe
fn require_account(tx: &Tx<'_>, account: &AccountId) -> Result<(), MiraError> {
    match tx.get_account(account)? {
        Some(_) => Ok(()),
        None => Err(account_not_found(account)),
    }
}

/// Action neuve, en attente, avec un identifiant frais
fn candidate_action(
    ctx: &QueueContext<'_>,
    request: &SubmitRequest,
    not_before: Timestamp,
) -> PendingAction {
    PendingAction {
        id: ActionId::from_raw(ctx.ids.next_id()),
        account: request.account.clone(),
        kind: request.kind.clone(),
        targets: request.targets.clone(),
        idempotency_key: request.key.clone(),
        state: ActionState::Pending,
        attempts: 0,
        last_error: None,
        created_at: ctx.clock.now(),
        not_before,
    }
}
