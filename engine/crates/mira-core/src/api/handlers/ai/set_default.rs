//! Désignation de la connexion d'IA par défaut.

use mira_vocabulary::{AiConnectionId, MiraError};

use super::connection_view::{read_connection, single};
use crate::api::contract::Response;
use crate::api::engine::Engine;

/// Désigne cette connexion par défaut en retirant l'ancienne ; inconnue : `not_found.ai_connection`
pub(crate) fn handle_set_default_ai_connection(
    engine: &Engine,
    id: &AiConnectionId,
) -> Result<Response, MiraError> {
    engine
        .database
        .write(|tx| tx.set_default_ai_connection(id))?;
    Ok(single(read_connection(engine, id)?))
}
