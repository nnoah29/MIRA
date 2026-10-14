//! Changements d'état d'un brouillon, dans la transaction de l'appelant.

use mira_vocabulary::{DraftId, DraftState, MiraError, Timestamp};

use crate::actions::errors::not_found::draft_not_found;
use crate::storage::Tx;

/// Fixe l'état d'un brouillon connu
pub(crate) fn set_draft_state(
    tx: &Tx<'_>,
    id: &DraftId,
    state: DraftState,
) -> Result<(), MiraError> {
    let mut draft = tx.get_draft(id)?.ok_or_else(|| draft_not_found(id))?;
    draft.state = state;
    tx.update_draft(&draft)
}

/// Replanifie le brouillon à `at` s'il existe encore ; un brouillon disparu est ignoré
pub(crate) fn schedule_if_present(
    tx: &Tx<'_>,
    id: &DraftId,
    at: Timestamp,
) -> Result<(), MiraError> {
    match tx.get_draft(id)? {
        Some(_) => set_draft_state(tx, id, DraftState::Scheduled { at }),
        None => Ok(()),
    }
}

/// Remet le brouillon en édition s'il existe encore ; un brouillon disparu est ignoré
pub(crate) fn mark_editing(tx: &Tx<'_>, id: &DraftId) -> Result<(), MiraError> {
    match tx.get_draft(id)? {
        Some(draft) if draft.state != DraftState::Editing => {
            set_draft_state(tx, id, DraftState::Editing)
        }
        Some(_) | None => Ok(()),
    }
}
