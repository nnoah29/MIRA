//! Ajout d'une pièce jointe lue depuis un fichier local du client.

use std::path::Path;

use mira_vocabulary::{DraftId, MiraError};

use crate::api::contract::{Response, WriteResponse};
use crate::api::engine::Engine;
use crate::drafts::attach_file;

/// Joint le fichier (25 Mo au plus) au brouillon en édition
pub(crate) fn handle_attach_file(
    engine: &Engine,
    draft: &DraftId,
    path: &str,
) -> Result<Response, MiraError> {
    let updated = attach_file(&engine.queue_context(), draft, Path::new(path))?;
    Ok(Response::Write(WriteResponse::Draft(Box::new(updated))))
}
