//! Suppression d'une connexion d'IA et de sa clé ; aucune autre connexion ne devient défaut.

use mira_vocabulary::{AiConnectionId, MiraError};

use super::connection_view::{done, key_location};
use crate::api::contract::Response;
use crate::api::engine::Engine;

/// Supprime la connexion puis sa clé du trousseau ; inconnue : `not_found.ai_connection`
pub(crate) fn handle_remove_ai_connection(
    engine: &Engine,
    id: &AiConnectionId,
) -> Result<Response, MiraError> {
    engine.database.write(|tx| tx.delete_ai_connection(id))?;
    engine.config.secrets.delete(&key_location(engine, id))?;
    Ok(done())
}
