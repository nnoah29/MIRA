//! Début d'une autorisation OAuth2 pour une adresse (15.2, S1).

use mira_vocabulary::{AccountId, EmailAddress, MiraError, ProviderId};

use super::begin_flow::begin_flow;
use crate::accounts::choose_provider;
use crate::api::contract::Response;
use crate::api::engine::Engine;

/// Réserve un identifiant de compte, ouvre le flux et rend son adresse d'autorisation
pub(crate) async fn handle_start_oauth(
    engine: &Engine,
    address: &str,
    requested: Option<&ProviderId>,
) -> Result<Response, MiraError> {
    let address = EmailAddress::parse(address)?;
    let provider = choose_provider(&engine.config.registry, &address, requested)?;
    let reserved = AccountId::from_raw(engine.config.ids.next_id());
    begin_flow(engine, &provider.descriptor(), &address, reserved, false).await
}
