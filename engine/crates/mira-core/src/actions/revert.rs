//! Annulation de l'effet local d'une action (échec définitif, annulation par l'utilisateur).

use mira_vocabulary::{ActionKind, MiraError, PendingAction};

use super::effect::apply_inverse;
use super::record::{load_record, save_record};
use crate::drafts::state::mark_editing;
use crate::storage::Tx;

/// Défait l'effet local de l'action (invariant 22) ; un envoi remet son brouillon en édition. Sans effet si déjà défait.
pub fn revert_local_effect(tx: &Tx<'_>, action: &PendingAction) -> Result<(), MiraError> {
    if let Some(mut record) = load_record(tx, &action.id)? {
        apply_inverse(tx, &record.inverse)?;
        record.inverse = Vec::new();
        save_record(tx, &action.id, &record)?;
    }
    if let ActionKind::Send { draft } = &action.kind {
        mark_editing(tx, draft)?;
    }
    Ok(())
}
