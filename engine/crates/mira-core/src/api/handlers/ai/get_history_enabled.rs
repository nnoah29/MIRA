//! Lecture de l'interrupteur de l'historique des conversations d'IA.

use mira_vocabulary::MiraError;

use crate::api::contract::{AiResponse, Response};
use crate::api::engine::Engine;

/// Rend l'interrupteur ; activé tant qu'il n'a jamais été réglé
pub(crate) fn handle_get_ai_history_enabled(engine: &Engine) -> Result<Response, MiraError> {
    let enabled = engine.database.read(|tx| tx.ai_history_enabled())?;
    Ok(Response::Ai(AiResponse::HistoryEnabled(enabled)))
}
