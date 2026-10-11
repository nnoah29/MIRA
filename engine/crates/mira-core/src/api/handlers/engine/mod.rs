//! Domaine « moteur ».

mod contract_info;
mod status;

use mira_vocabulary::MiraError;

use crate::api::contract::{EngineRequest, Response};
use crate::api::engine::Engine;

/// Route une requête du domaine moteur
pub(crate) async fn route(engine: &Engine, request: EngineRequest) -> Result<Response, MiraError> {
    match request {
        EngineRequest::GetContractInfo => contract_info::handle_get_contract_info(engine),
        EngineRequest::GetStatus => status::handle_get_status(engine),
    }
}
