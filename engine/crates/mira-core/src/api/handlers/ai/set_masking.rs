//! Réglage du masquage des données sensibles d'une connexion d'IA.

use mira_vocabulary::{AiConnectionId, MaskingSettings, MiraError};

use super::connection_view::{read_connection, single};
use crate::api::contract::Response;
use crate::api::engine::Engine;

/// Remplace le réglage du masquage ; inconnue : `not_found.ai_connection`
pub(crate) fn handle_set_ai_masking(
    engine: &Engine,
    id: &AiConnectionId,
    masking: MaskingSettings,
) -> Result<Response, MiraError> {
    engine
        .database
        .write(|tx| tx.set_ai_masking(id, &masking))?;
    Ok(single(read_connection(engine, id)?))
}
