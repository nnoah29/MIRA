//! Poses des libellés système, par conversation.

use mira_vocabulary::MiraError;

use crate::api::contract::{Response, SystemLabelResponse};
use crate::api::engine::Engine;

/// Rend les libellés système posés sur chaque conversation qui existe encore
pub(crate) fn handle_list_assignments(engine: &Engine) -> Result<Response, MiraError> {
    let listed = engine
        .database
        .read(|tx| tx.list_system_label_assignments())?;
    Ok(Response::SystemLabel(SystemLabelResponse::Assignments(
        listed,
    )))
}
