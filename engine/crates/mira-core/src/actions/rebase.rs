//! Rebase des intentions locales sur un miroir fraîchement synchronisé (D-23).

use mira_vocabulary::{AccountId, ActionState, MiraError};

use super::effect::apply_missing;
use super::plan::steps_for;
use super::record::{load_record, save_record};
use crate::storage::Tx;

/// Réapplique l'effet local des actions en attente ou en cours que le serveur a écrasé ; le serveur gagne ailleurs
pub fn reapply_pending_local_effects(tx: &Tx<'_>, account: &AccountId) -> Result<(), MiraError> {
    let active = [ActionState::Pending, ActionState::InProgress];
    for action in tx.list_actions(account, &active)? {
        let Some(mut record) = load_record(tx, &action.id)? else {
            continue;
        };
        let steps = steps_for(tx, &record.intent, &action.targets)?;
        let fresh = apply_missing(tx, &steps)?;
        if !fresh.is_empty() {
            record.inverse.splice(0..0, fresh);
            save_record(tx, &action.id, &record)?;
        }
    }
    Ok(())
}
