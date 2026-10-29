//! Consentement de l'utilisateur à envoyer du texte à une connexion d'IA.

use mira_vocabulary::{AiConnectionId, MiraError};

use super::connection_view::{read_connection, single};
use crate::api::contract::Response;
use crate::api::engine::Engine;

/// Note la date du consentement, une seule fois ; inconnue : `not_found.ai_connection`
pub(crate) fn handle_acknowledge_ai_consent(
    engine: &Engine,
    id: &AiConnectionId,
) -> Result<Response, MiraError> {
    let now = engine.config.clock.now().as_millis();
    engine.database.write(|tx| tx.set_ai_consent(id, now))?;
    Ok(single(read_connection(engine, id)?))
}
