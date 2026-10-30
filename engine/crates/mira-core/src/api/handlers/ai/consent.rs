//! Consentement de l'utilisateur à envoyer certains types de contenu à une connexion d'IA.

use mira_vocabulary::{AiConnectionId, ContentKind, MiraError};

use super::allowed_kinds::merged_kinds;
use super::connection_view::{read_connection, single};
use crate::api::contract::Response;
use crate::api::engine::Engine;

/// Note la date du consentement (la première seulement) et ajoute les types aux types autorisés ; inconnue : `not_found.ai_connection`
pub(crate) fn handle_acknowledge_ai_consent(
    engine: &Engine,
    id: &AiConnectionId,
    kinds: &[ContentKind],
) -> Result<Response, MiraError> {
    let now = engine.config.clock.now().as_millis();
    engine.database.write(|tx| {
        tx.set_ai_consent(id, now)?;
        let current = tx.require_ai_connection(id)?;
        tx.set_ai_allowed_kinds(id, &merged_kinds(&current.allowed_kinds, kinds))
    })?;
    Ok(single(read_connection(engine, id)?))
}
