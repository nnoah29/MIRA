//! Bornes des conversations d'IA enregistrées : titre en caractères, contenu en octets, et nombre gardé.

use mira_vocabulary::error::codes::invalid_input;
use mira_vocabulary::{ErrorCategory, MiraError};

pub(super) const MAX_TITLE_CHARS: usize = 80;
pub(super) const MAX_PAYLOAD_BYTES: usize = 2 * 1024 * 1024;
pub(super) const MAX_CONVERSATIONS: usize = 200;

/// Rend le titre rogné ; vide ou de plus de 80 caractères : `invalid_input.ai_conversation_title`
pub(super) fn validate_title(title: &str) -> Result<String, MiraError> {
    let trimmed = title.trim();
    match trimmed.chars().count() {
        1..=MAX_TITLE_CHARS => Ok(trimmed.to_string()),
        _ => Err(MiraError::new(
            ErrorCategory::InvalidInput,
            invalid_input::AI_CONVERSATION_TITLE,
            "le titre d'une conversation doit faire de 1 à 80 caractères",
        )),
    }
}

/// Vérifie la taille puis que le contenu est un objet JSON ; le moteur n'en lit pas davantage
pub(super) fn validate_payload(payload: &str) -> Result<(), MiraError> {
    if payload.len() > MAX_PAYLOAD_BYTES {
        return Err(MiraError::new(
            ErrorCategory::InvalidInput,
            invalid_input::AI_CONVERSATION_TOO_LARGE,
            "la conversation dépasse 2 Mio",
        ));
    }
    match serde_json::from_str::<serde_json::Value>(payload) {
        Ok(serde_json::Value::Object(_)) => Ok(()),
        _ => Err(MiraError::new(
            ErrorCategory::InvalidInput,
            invalid_input::AI_CONVERSATION_PAYLOAD,
            "le contenu d'une conversation doit être un objet JSON",
        )),
    }
}
