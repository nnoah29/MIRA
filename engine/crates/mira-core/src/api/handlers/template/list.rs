//! Liste des modèles de messages, par nom.

use mira_vocabulary::MiraError;

use crate::api::contract::{Response, TemplateResponse};
use crate::api::engine::Engine;

/// Rend tous les modèles, triés par nom sans tenir compte de la casse
pub(crate) fn handle_list_templates(engine: &Engine) -> Result<Response, MiraError> {
    let templates = engine.database.read(|tx| tx.list_templates())?;
    Ok(Response::Template(TemplateResponse::Templates(templates)))
}
