//! Adresse d'une connexion d'IA selon son fournisseur : obligatoire s'il est libre, refusée s'il est fixe.

use mira_vocabulary::{AiError, AiProviderDescriptor, MiraError};

use crate::ai::base_url::clean_base_url;

/// Adresse à garder pour ce fournisseur, nettoyée et contrôlée
pub(super) fn resolve_base_url(
    descriptor: &AiProviderDescriptor,
    base_url: Option<&str>,
) -> Result<Option<String>, MiraError> {
    match (descriptor.base_url_editable, base_url) {
        (true, Some(raw)) => clean_base_url(raw).map(Some),
        (false, None) => Ok(None),
        _ => Err(AiError::UrlInvalid.into()),
    }
}
