//! Rangement de la clé d'API d'une connexion d'IA dans le trousseau.

use mira_vocabulary::{AiConnectionId, MiraError};

use super::connection_view::{key_location, read_connection, single};
use super::validate_key::validate_key;
use crate::api::contract::{Response, SecretInput};
use crate::api::engine::Engine;

/// Contrôle puis range la clé ; elle n'est jamais renvoyée, la réponse porte seulement `has_key`
pub(crate) fn handle_set_ai_api_key(
    engine: &Engine,
    id: &AiConnectionId,
    key: SecretInput,
) -> Result<Response, MiraError> {
    read_connection(engine, id)?;
    let value = validate_key(key)?;
    engine
        .config
        .secrets
        .put(&key_location(engine, id), &value)?;
    Ok(single(read_connection(engine, id)?))
}
