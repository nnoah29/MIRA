//! Résolution du provider d'un compte existant (7.8).

use std::sync::Arc;

use mira_vocabulary::error::codes::not_found;
use mira_vocabulary::{ErrorCategory, MiraError, Provider, ProviderId};

use super::provider_registry::ProviderRegistry;

/// Retrouve le provider d'un compte, ou renvoie `not_found.provider` sans toucher aux données du compte.
pub fn resolve_account_provider(
    registry: &ProviderRegistry,
    id: &ProviderId,
) -> Result<Arc<dyn Provider>, MiraError> {
    registry.find(id).ok_or_else(|| {
        MiraError::new(
            ErrorCategory::NotFound,
            not_found::PROVIDER,
            format!("provider inconnu : {}", id.as_str()),
        )
    })
}
