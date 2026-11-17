//! Échec de l'échange du code : l'erreur OAuth du service est dite, jamais un secret.

use mira_vocabulary::error::codes::{authentication, server};
use mira_vocabulary::{ErrorCategory, MiraError};
use serde::Deserialize;

const MAX_DESCRIPTION: usize = 160;

#[derive(Deserialize)]
struct RawFailure {
    error: Option<String>,
    error_description: Option<String>,
}

/// Convertit la réponse d'échec : 5xx passager ; `invalid_grant` (code refusé, typiquement un abonnement non éligible) ; sinon refus générique ; l'erreur et sa description sont nommées
pub(super) fn exchange_failure(status: u16, body: &[u8], secrets: &[&str]) -> MiraError {
    if status >= 500 {
        let message = format!("service d'autorisation indisponible ({status})");
        return MiraError::new(ErrorCategory::Server, server::REJECTED, message).retryable(true);
    }
    let raw: Option<RawFailure> = serde_json::from_slice(body).ok();
    let reason = raw
        .as_ref()
        .and_then(|failure| failure.error.as_deref())
        .map(|text| clean(text, 64, &[]));
    let detail = raw
        .as_ref()
        .and_then(|failure| failure.error_description.as_deref())
        .map(|text| clean(text, MAX_DESCRIPTION, secrets))
        .filter(|text| !text.is_empty());
    let code = match reason.as_deref() {
        Some("invalid_grant") => authentication::AI_SESSION_INCOMPLETE,
        _ => authentication::REJECTED,
    };
    let mut message = format!("échange refusé par le service (HTTP {status}");
    for part in reason.into_iter().chain(detail) {
        message.push_str(&format!(", {part}"));
    }
    message.push(')');
    MiraError::new(ErrorCategory::Authentication, code, message)
}

/// Texte ASCII imprimable borné ; vide s'il recopie l'un des secrets
fn clean(text: &str, limit: usize, secrets: &[&str]) -> String {
    if secrets
        .iter()
        .any(|secret| !secret.is_empty() && text.contains(secret))
    {
        return String::new();
    }
    text.chars()
        .filter(|c| c.is_ascii_graphic() || *c == ' ')
        .take(limit)
        .collect()
}
