//! Liste des modèles d'une connexion d'IA, depuis le cache ou le service.

use mira_vocabulary::{AiConnectionId, MiraError};

use super::connection_view::read_connection;
use crate::ai::models::fetch_models;
use crate::api::contract::{AiResponse, Response};
use crate::api::engine::Engine;

/// Rend la liste gardée, ou interroge le service si `refresh` ou si rien n'est gardé
pub(crate) async fn handle_list_ai_models(
    engine: &Engine,
    id: &AiConnectionId,
    refresh: bool,
) -> Result<Response, MiraError> {
    let connection = read_connection(engine, id)?;
    let list = fetch_models(engine, &connection, refresh).await?;
    Ok(Response::Ai(AiResponse::Models(list)))
}
