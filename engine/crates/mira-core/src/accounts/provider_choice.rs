//! Choix du provider d'un compte et vérification de la méthode d'authentification demandée.

use std::sync::Arc;

use mira_vocabulary::error::codes::invalid_input;
use mira_vocabulary::{
    AuthMethodKind, EmailAddress, ErrorCategory, MiraError, Provider, ProviderDescriptor,
    ProviderId,
};

use crate::registry::{ProviderRegistry, resolve_account_provider};

/// Retient le provider demandé, ou celui que le registre détecte pour l'adresse
pub fn choose_provider(
    registry: &ProviderRegistry,
    address: &EmailAddress,
    requested: Option<&ProviderId>,
) -> Result<Arc<dyn Provider>, MiraError> {
    match requested {
        Some(id) => resolve_account_provider(registry, id),
        None => Ok(registry.detect(address)),
    }
}

/// Refuse une méthode d'authentification que le provider ne propose pas
pub fn require_auth_method(
    descriptor: &ProviderDescriptor,
    kind: AuthMethodKind,
) -> Result<(), MiraError> {
    if descriptor
        .auth_methods
        .iter()
        .any(|method| method.kind() == kind)
    {
        return Ok(());
    }
    Err(MiraError::new(
        ErrorCategory::InvalidInput,
        invalid_input::PARAMETER,
        format!(
            "le provider {} ne propose pas cette méthode d'authentification",
            descriptor.id.as_str()
        ),
    ))
}
