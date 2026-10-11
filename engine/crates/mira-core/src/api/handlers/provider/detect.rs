//! Détection du provider d'une adresse (6.6).

use mira_vocabulary::{EmailAddress, MiraError};

use crate::api::contract::Response;
use crate::api::engine::Engine;

/// Rend le provider détecté pour une adresse (repli générique si aucun ne la gère)
pub(crate) fn handle_detect_provider(
    engine: &Engine,
    address: &str,
) -> Result<Response, MiraError> {
    let address = EmailAddress::parse(address)?;
    Ok(Response::Provider(
        engine.config.registry.detect(&address).descriptor(),
    ))
}
