//! Reconstruction de l'index de recherche depuis les messages du miroir.

use mira_vocabulary::MiraError;

use crate::api::contract::Response;
use crate::api::engine::Engine;

/// Reconstruit tout l'index plein texte dans une seule transaction
pub(crate) fn handle_rebuild_index(engine: &Engine) -> Result<Response, MiraError> {
    engine.database.write(|tx| tx.rebuild_search_index())?;
    Ok(Response::Done)
}
