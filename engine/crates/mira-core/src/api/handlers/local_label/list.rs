//! Liste des libellés locaux, par nom.

use mira_vocabulary::MiraError;

use crate::api::contract::{LocalLabelResponse, Response};
use crate::api::engine::Engine;

/// Rend tous les libellés locaux, triés par nom sans tenir compte de la casse
pub(crate) fn handle_list_local_labels(engine: &Engine) -> Result<Response, MiraError> {
    let labels = engine.database.read(|tx| tx.list_local_labels())?;
    Ok(Response::LocalLabel(LocalLabelResponse::Labels(labels)))
}
