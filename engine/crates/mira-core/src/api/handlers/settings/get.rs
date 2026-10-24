//! Lecture des réglages du moteur.

use mira_vocabulary::MiraError;

use crate::api::contract::{Response, SettingsResponse};
use crate::api::engine::Engine;

/// Rend les réglages en vigueur
pub(crate) fn handle_get_settings(engine: &Engine) -> Result<Response, MiraError> {
    let settings = engine.database.read(|tx| tx.get_engine_settings())?;
    Ok(Response::Settings(SettingsResponse::Settings(settings)))
}
