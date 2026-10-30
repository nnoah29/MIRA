//! Liste des fournisseurs d'IA connus du moteur, dans l'ordre du registre.

use mira_vocabulary::MiraError;

use crate::api::contract::{AiResponse, Response};
use crate::api::engine::Engine;

/// Rend les fiches de tous les fournisseurs enregistrés
pub(crate) fn handle_list_ai_providers(engine: &Engine) -> Result<Response, MiraError> {
    let providers = engine.config.ai_registry.list();
    Ok(Response::Ai(AiResponse::Providers(providers)))
}
