//! Lecture de la réponse JSON du point d'accès de jetons (RFC 6749 section 5.1).

use mira_vocabulary::error::codes::server;
use mira_vocabulary::{ErrorCategory, MiraError};
use serde::Deserialize;

use super::set::TokenSet;
use crate::ports::{Clock, SecretValue};

#[derive(Deserialize)]
struct RawTokens {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: Option<u64>,
}

/// Convertit la réponse de succès en jetons ; l'expiration est calculée avec l'horloge du moteur
pub(super) fn parse_tokens(body: &[u8], clock: &dyn Clock) -> Result<TokenSet, MiraError> {
    let raw: RawTokens = serde_json::from_slice(body).map_err(|_| {
        MiraError::new(
            ErrorCategory::Server,
            server::PROTOCOL,
            "réponse de jetons illisible",
        )
    })?;
    let expires_at = raw.expires_in.map(|seconds| {
        let millis = i64::try_from(seconds).unwrap_or(i64::MAX / 1000);
        clock.now().plus_millis(millis.saturating_mul(1000))
    });
    Ok(TokenSet {
        access_token: SecretValue::new(raw.access_token),
        refresh_token: raw.refresh_token.map(SecretValue::new),
        expires_at,
    })
}

#[derive(Deserialize)]
struct RawKey {
    key: String,
}

/// Convertit la réponse de succès d'une connexion par navigateur en clé d'API, sans la copier ailleurs
pub(super) fn parse_key(body: &[u8]) -> Result<SecretValue, MiraError> {
    let raw: RawKey = serde_json::from_slice(body).map_err(|_| {
        MiraError::new(
            ErrorCategory::Server,
            server::PROTOCOL,
            "réponse de clé illisible",
        )
    })?;
    if raw.key.trim().is_empty() {
        return Err(MiraError::new(
            ErrorCategory::Server,
            server::PROTOCOL,
            "réponse de clé vide",
        ));
    }
    Ok(SecretValue::new(raw.key))
}
