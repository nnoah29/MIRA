//! Suppression d'un brouillon.

use mira_vocabulary::{DraftId, DraftState, MiraError};

use crate::actions::QueueContext;
use crate::actions::errors::incompatible::draft_not_editable;
use crate::actions::errors::not_found::draft_not_found;

/// Supprime un brouillon en édition avec ses pièces jointes ; annulez d'abord l'envoi d'un brouillon planifié
pub fn delete_draft(ctx: &QueueContext<'_>, id: &DraftId) -> Result<(), MiraError> {
    ctx.db.write(|tx| {
        let draft = tx.get_draft(id)?.ok_or_else(|| draft_not_found(id))?;
        if draft.state != DraftState::Editing {
            return Err(draft_not_editable());
        }
        tx.delete_draft(id)
    })
}
