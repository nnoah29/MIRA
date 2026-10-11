//! Préparation d'un compte : provider retenu et serveurs proposés (S1–S3).

use mira_vocabulary::{EmailAddress, MiraError, ProviderId};

use crate::accounts::{choose_provider, resolve_servers};
use crate::api::contract::{AccountPreparation, Response};
use crate::api::engine::Engine;

/// Rend le provider retenu pour l'adresse et les serveurs qui seront contactés
pub(crate) fn handle_prepare_account(
    engine: &Engine,
    address: &str,
    requested: Option<&ProviderId>,
) -> Result<Response, MiraError> {
    let address = EmailAddress::parse(address)?;
    let provider = choose_provider(&engine.config.registry, &address, requested)?;
    let servers = resolve_servers(&engine.config.network, provider.as_ref(), &address, None);
    Ok(Response::AccountPreparation(AccountPreparation {
        provider: provider.descriptor(),
        servers: servers.connect,
    }))
}
