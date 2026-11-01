//! Interrupteur de l'historique des conversations d'IA ; couper n'efface aucune conversation.

use mira_vocabulary::MiraError;

use crate::api::contract::{AiResponse, Response};
use crate::api::engine::Engine;

/// Règle l'interrupteur et rend la valeur enregistrée
pub(crate) fn handle_set_ai_history_enabled(
    engine: &Engine,
    enabled: bool,
) -> Result<Response, MiraError> {
    engine
        .database
        .write(|tx| tx.set_ai_history_enabled(enabled))?;
    Ok(Response::Ai(AiResponse::HistoryEnabled(enabled)))
}
