//! Liste de tous les libellés avec leurs réglages.

use mira_vocabulary::MiraError;

use super::overview::load_overviews;
use crate::api::contract::{LabelSettingsResponse, Response};
use crate::api::engine::Engine;

/// Rend les libellés locaux puis ceux de chaque compte, dans l'ordre d'affichage
pub(crate) fn handle_list_labels(engine: &Engine) -> Result<Response, MiraError> {
    let labels = engine.database.read(load_overviews)?;
    Ok(Response::LabelSettings(LabelSettingsResponse::Labels(
        labels,
    )))
}
