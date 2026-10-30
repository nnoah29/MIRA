//! Vidage du journal des envois à un service d'IA.

use mira_vocabulary::MiraError;

use super::connection_view::done;
use crate::api::contract::Response;
use crate::api::engine::Engine;

/// Supprime toutes les lignes du journal ; un journal déjà vide n'est pas une erreur
pub(crate) fn handle_clear_ai_journal(engine: &Engine) -> Result<Response, MiraError> {
    engine.database.write(|tx| tx.clear_ai_journal())?;
    Ok(done())
}
