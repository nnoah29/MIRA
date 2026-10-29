//! Liste des connexions d'IA, avec leur indicateur de clé.

use mira_vocabulary::MiraError;

use super::connection_view::with_key_flag;
use crate::api::contract::{AiResponse, Response};
use crate::api::engine::Engine;

/// Rend les connexions triées par libellé, chacune avec `has_key` calculé depuis le trousseau
pub(crate) fn handle_list_ai_connections(engine: &Engine) -> Result<Response, MiraError> {
    let stored = engine.database.read(|tx| tx.list_ai_connections())?;
    let connections = stored
        .into_iter()
        .map(|connection| with_key_flag(engine, connection))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Response::Ai(AiResponse::Connections(connections)))
}
