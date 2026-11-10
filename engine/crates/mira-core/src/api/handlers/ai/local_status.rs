//! État du modèle local : lecture seule, rien n'est créé, téléchargé ni lancé.

use mira_vocabulary::MiraError;

use crate::ai::local::local_status;
use crate::api::contract::{AiResponse, Response};
use crate::api::engine::Engine;

/// Rend l'état du modèle local
pub(crate) async fn handle_get_local_runtime_status(
    engine: &Engine,
) -> Result<Response, MiraError> {
    Ok(Response::Ai(AiResponse::LocalRuntime(
        local_status(engine).await,
    )))
}
