//! Liste des providers disponibles.

use mira_vocabulary::MiraError;

use crate::api::contract::Response;
use crate::api::engine::Engine;

/// Rend le catalogue des providers, repli compris
pub(crate) fn handle_list_providers(engine: &Engine) -> Result<Response, MiraError> {
    Ok(Response::Providers(engine.config.registry.list()))
}
