//! Action remise en attente après un échec qui ne la condamne pas : réseau, jeton, limite de débit.

use mira_vocabulary::{AccountState, ActionKind, ActionState, MiraError, PendingAction, Timestamp};

use super::backoff::backoff_millis;
use super::context::ReplayContext;
use super::report::{ReplayStop, StepOutcome};
use crate::accounts::change_account_state;
use crate::drafts::state::schedule_if_present;
use crate::storage::Tx;

/// Remet l'action en attente avec un délai croissant, et arrête le rejeu pour garder l'ordre (inv. 19)
pub(super) fn park_transient(
    ctx: &ReplayContext<'_>,
    action: &PendingAction,
    error: &MiraError,
) -> Result<StepOutcome, MiraError> {
    let delay = backoff_millis(action.attempts + 1);
    let retry_at = ctx.queue.clock.now().plus_millis(delay);
    ctx.queue
        .db
        .write(|tx| requeue(tx, action, error, retry_at))?;
    Ok(StepOutcome::Stop(ReplayStop::Retry {
        code: error.code.clone(),
    }))
}

/// Garde l'action en file, passe le compte en « authentification requise » et arrête le rejeu (S10)
pub(super) fn park_for_authentication(
    ctx: &ReplayContext<'_>,
    action: &PendingAction,
    error: &MiraError,
) -> Result<StepOutcome, MiraError> {
    let now = ctx.queue.clock.now();
    ctx.queue.db.write(|tx| requeue(tx, action, error, now))?;
    require_reauthentication(ctx)?;
    Ok(StepOutcome::Stop(ReplayStop::AuthRequired))
}

/// Passe le compte en « authentification requise » s'il ne l'est pas déjà
pub(super) fn require_reauthentication(ctx: &ReplayContext<'_>) -> Result<(), MiraError> {
    let account = ctx.queue.db.read(|tx| tx.get_account(ctx.account))?;
    match account {
        Some(found) if found.state != AccountState::AuthRequired => change_account_state(
            ctx.queue.db,
            ctx.queue.events,
            found,
            AccountState::AuthRequired,
        )
        .map(|_| ()),
        Some(_) | None => Ok(()),
    }
}

/// Note l'échec, remet l'action en attente à `retry_at` et rend un envoi à son brouillon planifié
fn requeue(
    tx: &Tx<'_>,
    action: &PendingAction,
    error: &MiraError,
    retry_at: Timestamp,
) -> Result<(), MiraError> {
    tx.record_action_failure(&action.id, &error.to_string(), retry_at)?;
    tx.set_action_state(&action.id, ActionState::Pending)?;
    if let ActionKind::Send { draft } = &action.kind {
        schedule_if_present(tx, draft, retry_at)?;
    }
    Ok(())
}
