//! En-têtes d'une requête vers un service d'IA : ceux du format, puis la clé ajoutée en dernier.

use mira_vocabulary::{AiError, AuthScheme};
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderName, HeaderValue};

use crate::ports::SecretValue;

const X_API_KEY: &str = "x-api-key";

/// Construit les en-têtes à partir des paires fournies par le format ou le descripteur.
pub(super) fn header_map(pairs: &[(String, String)]) -> Result<HeaderMap, AiError> {
    let mut headers = HeaderMap::new();
    for (name, value) in pairs {
        let name = HeaderName::from_bytes(name.as_bytes()).map_err(|_| AiError::BadResponse)?;
        let value = HeaderValue::from_str(value).map_err(|_| AiError::BadResponse)?;
        headers.append(name, value);
    }
    Ok(headers)
}

/// Ajoute la clé selon le schéma du descripteur, valeur marquée sensible ; une clé illisible est refusée.
pub(super) fn add_auth_header(
    headers: &mut HeaderMap,
    scheme: AuthScheme,
    key: &SecretValue,
) -> Result<(), AiError> {
    let (name, text) = match scheme {
        AuthScheme::Bearer => (AUTHORIZATION, format!("Bearer {}", key.expose())),
        AuthScheme::XApiKey => (HeaderName::from_static(X_API_KEY), key.expose().to_string()),
    };
    let mut value = HeaderValue::from_str(&text).map_err(|_| AiError::KeyRejected)?;
    value.set_sensitive(true);
    headers.insert(name, value);
    Ok(())
}
