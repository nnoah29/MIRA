//! Retrait de la clé d'API d'une connexion d'IA.

use mira_vocabulary::{AiConnectionId, MiraError};

use super::connection_view::{key_location, read_connection, single};
use crate::api::contract::Response;
use crate::api::engine::Engine;

/// Retire la clé du trousseau (après un éventuel rafraîchissement en cours) ; une clé absente n'est pas une erreur
pub(crate) async fn handle_clear_ai_api_key(
    engine: &Engine,
    id: &AiConnectionId,
) -> Result<Response, MiraError> {
    read_connection(engine, id)?;
    let lock = engine.ai_session_locks.of(id);
    let _renewing = lock.lock().await;
    engine.config.secrets.delete(&key_location(engine, id))?;
    Ok(single(read_connection(engine, id)?))
}
