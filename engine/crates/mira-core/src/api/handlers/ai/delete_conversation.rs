//! Suppression d'une conversation enregistrée.

use mira_vocabulary::MiraError;

use super::connection_view::done;
use crate::api::contract::Response;
use crate::api::engine::Engine;

/// Supprime la conversation ; inconnue : `not_found.ai_conversation`
pub(crate) fn handle_delete_ai_conversation(
    engine: &Engine,
    id: &str,
) -> Result<Response, MiraError> {
    engine.database.write(|tx| tx.delete_ai_conversation(id))?;
    Ok(done())
}
