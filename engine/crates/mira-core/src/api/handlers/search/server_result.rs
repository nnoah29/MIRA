//! Lecture du résultat d'une recherche serveur terminée.

use mira_vocabulary::{MiraError, OperationId};

use crate::api::contract::Response;
use crate::api::engine::Engine;

/// Rend les messages trouvés par le serveur pour cette opération
pub(crate) fn handle_get_server_search_result(
    engine: &Engine,
    operation: &OperationId,
) -> Result<Response, MiraError> {
    Ok(Response::ServerSearch(
        engine.server_searches.get(operation)?,
    ))
}
