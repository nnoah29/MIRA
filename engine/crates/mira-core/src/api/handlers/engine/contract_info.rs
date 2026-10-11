//! Version du contrat et capacités du moteur.

use mira_vocabulary::{CONTRACT_VERSION, MiraError};

use crate::api::contract::{ContractInfo, Response};
use crate::api::engine::Engine;

const ENGINE_CAPABILITIES: &[&str] = &[
    "imap",
    "smtp",
    "oauth2",
    "app_password",
    "fts5",
    "safe_html",
];

/// Rend la version du contrat et la liste des capacités du moteur
pub(crate) fn handle_get_contract_info(_engine: &Engine) -> Result<Response, MiraError> {
    Ok(Response::ContractInfo(ContractInfo {
        version: CONTRACT_VERSION,
        engine_capabilities: ENGINE_CAPABILITIES
            .iter()
            .map(|c| (*c).to_string())
            .collect(),
    }))
}
