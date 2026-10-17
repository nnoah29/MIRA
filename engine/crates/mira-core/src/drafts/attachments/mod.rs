//! Pièces jointes d'un brouillon : ajout depuis un fichier local, retrait.

mod local_file;
mod mime_type;

use std::path::Path;

use mira_vocabulary::{Draft, DraftId, DraftState, MiraError};

use crate::actions::QueueContext;
use crate::actions::errors::incompatible::draft_not_editable;
use crate::actions::errors::input::invalid_parameter;
use crate::actions::errors::not_found::draft_not_found;
use crate::storage::Tx;
pub use local_file::MAX_ATTACHMENT_BYTES;
use local_file::read_local_file;

/// Joint un fichier local (25 Mo au plus) au brouillon en édition ; seul le nom du fichier est gardé, pas son chemin
pub fn attach_file(
    ctx: &QueueContext<'_>,
    draft: &DraftId,
    path: &Path,
) -> Result<Draft, MiraError> {
    let file = read_local_file(path)?;
    ctx.db.write(|tx| {
        require_editing(tx, draft)?;
        tx.add_draft_attachment(draft, &file.filename, &file.mime_type, &file.data)?;
        touch(tx, ctx, draft)
    })
}

/// Retire la pièce jointe d'après son rang dans la liste du brouillon (0 pour la première)
pub fn remove_attachment(
    ctx: &QueueContext<'_>,
    draft: &DraftId,
    index: usize,
) -> Result<Draft, MiraError> {
    ctx.db.write(|tx| {
        require_editing(tx, draft)?;
        let positions = tx.list_draft_attachment_positions(draft)?;
        let position = positions
            .get(index)
            .ok_or_else(|| invalid_parameter("rang de pièce jointe inconnu"))?;
        tx.remove_draft_attachment(draft, *position)?;
        touch(tx, ctx, draft)
    })
}

/// Exige un brouillon connu en cours d'édition
fn require_editing(tx: &Tx<'_>, id: &DraftId) -> Result<(), MiraError> {
    let draft = tx.get_draft(id)?.ok_or_else(|| draft_not_found(id))?;
    if draft.state != DraftState::Editing {
        return Err(draft_not_editable());
    }
    Ok(())
}

/// Date la modification et relit le brouillon avec ses pièces jointes
fn touch(tx: &Tx<'_>, ctx: &QueueContext<'_>, id: &DraftId) -> Result<Draft, MiraError> {
    let mut draft = tx.get_draft(id)?.ok_or_else(|| draft_not_found(id))?;
    draft.updated_at = ctx.clock.now();
    tx.update_draft(&draft)?;
    Ok(draft)
}
