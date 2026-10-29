//! Adresse de base saisie pour une connexion d'IA : nettoyée et contrôlée avant d'être gardée.

use mira_vocabulary::{AiError, MiraError};
use reqwest::Url;

use crate::auth::loopback::require_secure_url;

const MAX_URL_BYTES: usize = 2048;

/// Retire blancs et `/` de fin, refuse identifiants, requête, fragment et clair hors boucle locale
pub(crate) fn clean_base_url(raw: &str) -> Result<String, MiraError> {
    let cleaned = raw.trim().trim_end_matches('/');
    let parsed = Url::parse(cleaned).map_err(|_| AiError::UrlInvalid)?;
    let tidy = parsed.username().is_empty()
        && parsed.password().is_none()
        && parsed.query().is_none()
        && parsed.fragment().is_none()
        && cleaned.len() <= MAX_URL_BYTES;
    if !tidy {
        return Err(AiError::UrlInvalid.into());
    }
    require_secure_url(cleaned).map_err(|_| AiError::UrlInvalid)?;
    Ok(cleaned.to_string())
}
