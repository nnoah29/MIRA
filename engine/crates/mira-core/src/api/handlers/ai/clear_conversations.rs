//! Effacement de toutes les conversations enregistrées ; le journal des envois n'est pas touché.

use mira_vocabulary::MiraError;

use super::connection_view::done;
use crate::api::contract::Response;
use crate::api::engine::Engine;

/// Supprime toutes les conversations ; une liste déjà vide n'est pas une erreur
pub(crate) fn handle_clear_ai_conversations(engine: &Engine) -> Result<Response, MiraError> {
    engine.database.write(|tx| tx.clear_ai_conversations())?;
    Ok(done())
}
