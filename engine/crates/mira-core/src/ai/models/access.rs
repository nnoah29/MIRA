//! Ce qu'il faut pour joindre le service d'une connexion : fournisseur, adresse de base et clé.

use std::sync::Arc;

use mira_vocabulary::error::codes::{incompatible_state, not_found};
use mira_vocabulary::{
    AiConnection, AiError, AiProvider, AiProviderDescriptor, AiSecretRef, ErrorCategory, MiraError,
};

use crate::api::Engine;
use crate::auth::loopback::is_loopback_host;
use crate::ports::SecretValue;

/// Retrouve le fournisseur de la connexion dans le registre, sinon `not_found.ai_provider`.
pub(super) fn find_provider(
    engine: &Engine,
    connection: &AiConnection,
) -> Result<Arc<dyn AiProvider>, MiraError> {
    engine
        .config
        .ai_registry
        .find(&connection.provider)
        .ok_or_else(|| {
            MiraError::new(
                ErrorCategory::NotFound,
                not_found::AI_PROVIDER,
                "fournisseur d'IA inconnu",
            )
        })
}

/// Adresse de base : celle de la connexion si elle en a une, sinon celle du fournisseur.
pub(super) fn base_url_of(connection: &AiConnection, descriptor: &AiProviderDescriptor) -> String {
    connection
        .base_url
        .clone()
        .unwrap_or_else(|| descriptor.base_url.clone())
}

/// Lit la clé du trousseau ; absente, elle n'est tolérée que vers la boucle locale (clé vide).
pub(super) fn key_for(
    engine: &Engine,
    connection: &AiConnection,
    base_url: &str,
) -> Result<SecretValue, MiraError> {
    let location = AiSecretRef {
        profile: engine.config.profile.clone(),
        connection: connection.id.clone(),
    };
    match engine.config.secrets.get(&location)? {
        Some(key) => Ok(key),
        None if is_loopback_url(base_url)? => Ok(SecretValue::new("")),
        None => Err(MiraError::new(
            ErrorCategory::IncompatibleState,
            incompatible_state::AI_KEY_MISSING,
            "aucune clé d'API pour cette connexion d'IA",
        )),
    }
}

/// Dit si l'adresse vise la machine locale ; une adresse illisible est `invalid_input.ai_url_invalid`.
fn is_loopback_url(base_url: &str) -> Result<bool, MiraError> {
    let url = reqwest::Url::parse(base_url).map_err(|_| AiError::UrlInvalid)?;
    Ok(url.host_str().is_some_and(is_loopback_host))
}
