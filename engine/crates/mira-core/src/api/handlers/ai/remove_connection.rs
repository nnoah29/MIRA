//! Suppression d'une connexion d'IA et de sa clé ; aucune autre connexion ne devient défaut.

use mira_vocabulary::{AiConnectionId, MiraError};

use super::connection_view::{done, key_location};
use crate::api::contract::Response;
use crate::api::engine::Engine;

/// Supprime la clé du trousseau puis la connexion, pour ne jamais laisser de clé orpheline ; inconnue : `not_found.ai_connection`
pub(crate) async fn handle_remove_ai_connection(
    engine: &Engine,
    id: &AiConnectionId,
) -> Result<Response, MiraError> {
    engine.ai.key_logins.cancel_connection(id);
    let lock = engine.ai.session_locks.of(id);
    let _renewing = lock.lock().await;
    engine.config.secrets.delete(&key_location(engine, id))?;
    engine.database.write(|tx| tx.delete_ai_connection(id))?;
    Ok(done())
}
