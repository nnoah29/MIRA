//! Relance d'une action échouée : l'effet local revient et l'action repart en attente.

use mira_vocabulary::{ActionId, ActionKind, ActionState, DraftState, EventKind, MiraError};

use super::effect::apply_all;
use super::errors::incompatible::action_not_retryable;
use super::errors::not_found::action_not_found;
use super::model::QueueContext;
use super::plan::steps_for;
use super::record::{load_record, save_record};
use crate::drafts::state::set_draft_state;

/// Remet une action échouée en attente, immédiatement rejouable, et réapplique son effet local
pub fn retry(ctx: &QueueContext<'_>, id: &ActionId) -> Result<(), MiraError> {
    let now = ctx.clock.now();
    let account = ctx.db.write(|tx| {
        let action = tx.get_action(id)?.ok_or_else(|| action_not_found(id))?;
        if action.state != ActionState::Failed {
            return Err(action_not_retryable(id));
        }
        if let Some(mut record) = load_record(tx, id)? {
            let steps = steps_for(tx, &record.intent, &action.targets)?;
            record.inverse = apply_all(tx, &steps)?;
            save_record(tx, id, &record)?;
        }
        if let ActionKind::Send { draft } = &action.kind {
            set_draft_state(tx, draft, DraftState::Scheduled { at: now })?;
        }
        tx.requeue_action(id, now)?;
        Ok(action.account)
    })?;
    ctx.events.emit(EventKind::ActionQueued {
        account,
        action: id.clone(),
    })?;
    Ok(())
}
