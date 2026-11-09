//! Annulation d'une connexion par navigateur : l'écouteur est fermé, rien n'est rangé.

use mira_vocabulary::MiraError;

use super::connection_view::done;
use crate::api::contract::Response;
use crate::api::engine::Engine;

/// Annule le flux ; un flux inconnu ou déjà annulé n'est pas une erreur
pub(crate) fn handle_cancel_ai_key_login(
    engine: &Engine,
    flow: &str,
) -> Result<Response, MiraError> {
    engine.ai_key_logins.cancel(flow);
    Ok(done())
}
