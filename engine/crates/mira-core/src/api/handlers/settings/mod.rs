//! Domaine « réglages du moteur » : lecture et remplacement des réglages appliqués par le moteur.

mod get;
mod set;

use mira_vocabulary::MiraError;

use crate::api::contract::{Response, SettingsRequest, SettingsRequest as R};
use crate::api::engine::Engine;

/// Route une requête du domaine réglages du moteur
pub(crate) async fn route(
    engine: &Engine,
    request: SettingsRequest,
) -> Result<Response, MiraError> {
    match request {
        R::GetSettings => get::handle_get_settings(engine),
        R::SetSettings { settings } => set::handle_set_settings(engine, &settings),
    }
}
