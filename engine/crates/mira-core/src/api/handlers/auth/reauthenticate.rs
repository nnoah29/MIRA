//! Nouvelle autorisation OAuth2 d'un compte dont les jetons sont révoqués (7.7, S10).

use mira_vocabulary::error::codes::invalid_input;
use mira_vocabulary::{AccountId, AuthMethodKind, ErrorCategory, MiraError};

use super::begin_flow::begin_flow;
use crate::accounts::require_active_account;
use crate::api::contract::Response;
use crate::api::engine::Engine;
use crate::registry::resolve_account_provider;

/// Ouvre un flux qui remplacera les jetons du compte existant
pub(crate) async fn handle_reauthenticate(
    engine: &Engine,
    id: &AccountId,
) -> Result<Response, MiraError> {
    let account = engine.database.read(|tx| require_active_account(tx, id))?;
    if account.auth != AuthMethodKind::OAuth2 {
        return Err(MiraError::new(
            ErrorCategory::InvalidInput,
            invalid_input::PARAMETER,
            "ce compte n'utilise pas OAuth2 : fournir un nouveau mot de passe",
        ));
    }
    let provider = resolve_account_provider(&engine.config.registry, &account.provider)?;
    let descriptor = provider.descriptor();
    begin_flow(engine, &descriptor, &account.address, account.id, true).await
}
