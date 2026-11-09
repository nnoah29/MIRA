//! Réponse du point de jeton d'une session : jetons, échéance, portées accordées.

use mira_vocabulary::error::codes::server;
use mira_vocabulary::{ErrorCategory, MiraError};
use serde::Deserialize;

/// Réponse de succès, telle que le service la rend.
#[derive(Deserialize)]
pub(super) struct Grant {
    pub(super) access_token: String,
    pub(super) refresh_token: Option<String>,
    pub(super) id_token: Option<String>,
    pub(super) expires_in: Option<u64>,
    pub(super) scope: Option<String>,
}

/// Lit la réponse ; un corps illisible ou sans jeton d'accès est une erreur de protocole
pub(super) fn parse_grant(body: &[u8]) -> Result<Grant, MiraError> {
    let grant: Grant =
        serde_json::from_slice(body).map_err(|_| protocol("réponse de jetons illisible"))?;
    if grant.access_token.trim().is_empty() {
        return Err(protocol("réponse de jetons sans jeton d'accès"));
    }
    Ok(grant)
}

/// Dit si la portée accordée contient celle qu'on exige
pub(super) fn has_scope(granted: Option<&str>, required: &str) -> bool {
    let list = granted.unwrap_or_default();
    list.split([' ', '+']).any(|scope| scope == required)
}

/// Échéance en secondes Unix, une heure par défaut quand le service n'en donne pas
pub(super) fn expiry(grant: &Grant, now: i64) -> i64 {
    let seconds = grant.expires_in.unwrap_or(3600);
    now.saturating_add(i64::try_from(seconds).unwrap_or(i64::MAX / 2))
}

/// Erreur de protocole sans valeur issue de la réponse
fn protocol(message: &str) -> MiraError {
    MiraError::new(ErrorCategory::Server, server::PROTOCOL, message)
}
