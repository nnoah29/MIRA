//! Domaine « providers ».

mod detect;
mod list;

use mira_vocabulary::MiraError;

use crate::api::contract::{ProviderRequest, Response};
use crate::api::engine::Engine;

/// Route une requête du domaine providers
pub(crate) async fn route(
    engine: &Engine,
    request: ProviderRequest,
) -> Result<Response, MiraError> {
    match request {
        ProviderRequest::ListProviders => list::handle_list_providers(engine),
        ProviderRequest::DetectProvider { address } => {
            detect::handle_detect_provider(engine, &address)
        }
    }
}
