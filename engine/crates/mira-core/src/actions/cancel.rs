//! Annulation d'une action encore en attente (undo).

use mira_vocabulary::{ActionId, ActionState, EventKind, MiraError};

use super::errors::incompatible::action_not_cancellable;
use super::errors::not_found::action_not_found;
use super::model::QueueContext;
use super::revert::revert_local_effect;

/// Annule l'action tant qu'elle est en attente : effet local défait, état « annulé » ; sinon `action_not_cancellable`
pub fn cancel(ctx: &QueueContext<'_>, id: &ActionId) -> Result<(), MiraError> {
    let account = ctx.db.write(|tx| {
        let action = tx.get_action(id)?.ok_or_else(|| action_not_found(id))?;
        if action.state != ActionState::Pending {
            return Err(action_not_cancellable(id));
        }
        revert_local_effect(tx, &action)?;
        tx.set_action_state(id, ActionState::Cancelled)?;
        Ok(action.account)
    })?;
    ctx.events.emit(EventKind::ActionCancelled {
        account,
        action: id.clone(),
    })?;
    Ok(())
}
