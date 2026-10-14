//! Modification du contenu d'un brouillon en cours d'édition.

use mira_vocabulary::{DraftId, MiraError};

use crate::api::contract::{DraftChanges, Response, WriteResponse};
use crate::api::engine::Engine;
use crate::drafts::{DraftEdit, edit_draft};

/// Applique les champs présents et rend le brouillon à jour
pub(crate) fn handle_update_draft(
    engine: &Engine,
    draft: &DraftId,
    changes: DraftChanges,
) -> Result<Response, MiraError> {
    let updated = edit_draft(&engine.queue_context(), draft, edit_of(changes))?;
    Ok(Response::Write(WriteResponse::Draft(Box::new(updated))))
}

/// Traduit les changements du contrat en modification de brouillon ; un HTML vide efface la version HTML
fn edit_of(changes: DraftChanges) -> DraftEdit {
    DraftEdit {
        identity: changes.identity,
        to: changes.to,
        cc: changes.cc,
        bcc: changes.bcc,
        subject: changes.subject,
        body_text: changes.body_text,
        body_html: changes
            .body_html
            .map(|html| if html.is_empty() { None } else { Some(html) }),
    }
}
