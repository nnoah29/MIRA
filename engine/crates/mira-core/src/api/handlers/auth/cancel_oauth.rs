//! Annulation d'un flux OAuth2 (15.2) : l'écouteur est fermé et les jetons déjà obtenus sont effacés.

use mira_vocabulary::MiraError;

use crate::api::contract::Response;
use crate::api::engine::Engine;

/// Annule le flux ; un flux inconnu ou déjà annulé n'est pas une erreur
pub(crate) fn handle_cancel_oauth(engine: &Engine, flow: &str) -> Result<Response, MiraError> {
    if let Some(summary) = engine.oauth_flows.cancel(flow) {
        let orphan_tokens = summary.completed && !summary.target.reauthentication;
        if orphan_tokens {
            let profile = &engine.config.profile;
            engine
                .vault()
                .delete_all(profile, &summary.target.account)?;
        }
    }
    Ok(Response::Done)
}
