//! Poses de libellés locaux, par conversation.

use mira_vocabulary::MiraError;

use crate::api::contract::{LocalLabelResponse, Response};
use crate::api::engine::Engine;

/// Rend les libellés posés sur chaque conversation qui existe encore
pub(crate) fn handle_list_assignments(engine: &Engine) -> Result<Response, MiraError> {
    let listed = engine.database.read(|tx| tx.list_label_assignments())?;
    Ok(Response::LocalLabel(LocalLabelResponse::Assignments(
        listed,
    )))
}
