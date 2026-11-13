//! Rafraîchissement d'une session : jetons de remplacement, ou échec permanent ou passager.

use mira_vocabulary::AiSessionLogin;

use super::bundle::AiSession;
use super::grant::{expiry, parse_grant};
use crate::auth::tokens::post_form;

/// Échec d'un rafraîchissement : la session est perdue, ou l'essai peut se répéter plus tard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionRefreshError {
    Permanent,
    Transient,
}

/// Présente le jeton de rafraîchissement ; le jeton de remplacement de la réponse est obligatoire
pub async fn refresh_session(
    login: &AiSessionLogin,
    session: &AiSession,
    now: i64,
) -> Result<AiSession, SessionRefreshError> {
    let pairs = [
        ("grant_type", "refresh_token"),
        ("client_id", session.client_id.as_str()),
        ("refresh_token", session.refresh_token.as_str()),
        ("resource", login.resource.as_str()),
    ];
    let (status, body) = post_form(&login.token_url, &pairs)
        .await
        .map_err(|_| SessionRefreshError::Transient)?;
    if !(200..300).contains(&status) {
        return Err(classify(status));
    }
    let grant = parse_grant(&body).map_err(|_| SessionRefreshError::Transient)?;
    let refresh_token = grant
        .refresh_token
        .clone()
        .ok_or(SessionRefreshError::Transient)?;
    Ok(AiSession {
        access_token: grant.access_token.clone(),
        refresh_token,
        expires_at: expiry(&grant, now),
        ..session.clone()
    })
}

/// Classe un échec : 408, 429 et 5xx passent, tout autre refus du service est permanent
fn classify(status: u16) -> SessionRefreshError {
    if status == 408 || status == 429 || status >= 500 {
        SessionRefreshError::Transient
    } else {
        SessionRefreshError::Permanent
    }
}
