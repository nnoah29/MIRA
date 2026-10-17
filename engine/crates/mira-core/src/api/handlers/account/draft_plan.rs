//! Analyse d'un compte à ajouter ou à tester : adresse, provider, méthode et serveurs.

use std::sync::Arc;

use mira_vocabulary::error::codes::invalid_input;
use mira_vocabulary::{AuthMethodKind, EmailAddress, ErrorCategory, MiraError, Provider};

use crate::accounts::{ResolvedServers, choose_provider, require_auth_method, resolve_servers};
use crate::api::contract::{AccountDraft, CredentialInput};
use crate::api::engine::Engine;

/// Ce que le brouillon de compte devient une fois validé.
#[derive(Debug, Clone)]
pub(super) struct DraftPlan {
    pub address: EmailAddress,
    pub provider: Arc<dyn Provider>,
    pub servers: ResolvedServers,
    pub auth: AuthMethodKind,
}

/// Valide le brouillon : adresse, provider, concordance du secret et serveurs
pub(super) fn plan_draft(
    engine: &Engine,
    draft: &AccountDraft,
    credential: &CredentialInput,
) -> Result<DraftPlan, MiraError> {
    let address = EmailAddress::parse(&draft.address)?;
    let provider = choose_provider(&engine.config.registry, &address, draft.provider.as_ref())?;
    if credential.auth_kind() != draft.auth {
        return Err(MiraError::new(
            ErrorCategory::InvalidInput,
            invalid_input::PARAMETER,
            "le secret fourni ne correspond pas à la méthode d'authentification choisie",
        ));
    }
    require_auth_method(&provider.descriptor(), draft.auth)?;
    let servers = resolve_servers(
        &engine.config.network,
        provider.as_ref(),
        &address,
        draft.servers.clone(),
    );
    Ok(DraftPlan {
        address,
        provider,
        servers,
        auth: draft.auth,
    })
}
