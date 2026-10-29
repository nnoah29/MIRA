//! Lecture d'une connexion telle qu'on la rend au client : `has_key` calculé depuis le trousseau.

use mira_vocabulary::{AiConnection, AiConnectionId, AiSecretRef, MiraError};

use crate::api::contract::{AiResponse, Response};
use crate::api::engine::Engine;

/// Emplacement de la clé d'API de la connexion dans le trousseau
pub(super) fn key_location(engine: &Engine, id: &AiConnectionId) -> AiSecretRef {
    AiSecretRef {
        profile: engine.config.profile.clone(),
        connection: id.clone(),
    }
}

/// Renseigne `has_key` d'après le trousseau
pub(super) fn with_key_flag(
    engine: &Engine,
    connection: AiConnection,
) -> Result<AiConnection, MiraError> {
    let location = key_location(engine, &connection.id);
    let has_key = engine.config.secrets.get(&location)?.is_some();
    Ok(AiConnection {
        has_key,
        ..connection
    })
}

/// Relit la connexion en base, à l'instant, avec `has_key` ; inconnue : `not_found.ai_connection`
pub(super) fn read_connection(
    engine: &Engine,
    id: &AiConnectionId,
) -> Result<AiConnection, MiraError> {
    let stored = engine.database.read(|tx| tx.require_ai_connection(id))?;
    with_key_flag(engine, stored)
}

/// Réponse portant une seule connexion
pub(super) fn single(connection: AiConnection) -> Response {
    Response::Ai(AiResponse::Connection(connection))
}

/// Réponse sans contenu
pub(super) fn done() -> Response {
    Response::Ai(AiResponse::Done)
}
