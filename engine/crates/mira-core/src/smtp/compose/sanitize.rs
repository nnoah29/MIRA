//! Nettoyage des valeurs libres avant écriture dans un en-tête (anti-injection d'en-têtes).

use mira_vocabulary::error::codes::invalid_input;
use mira_vocabulary::{ErrorCategory, MiraError};

/// Remplace tout caractère de contrôle par une espace pour tenir sur une seule ligne
pub(super) fn single_line(text: &str) -> String {
    let cleaned: String = text
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect();
    cleaned.trim().to_string()
}

/// Retire les chevrons d'un identifiant de message et refuse les formes dangereuses
pub(super) fn bare_message_id(raw: &str) -> Result<String, MiraError> {
    let id = raw.trim().trim_start_matches('<').trim_end_matches('>');
    let forbidden = |c: char| c.is_control() || c.is_whitespace() || matches!(c, '<' | '>');
    if id.is_empty() || id.contains(forbidden) {
        return Err(MiraError::new(
            ErrorCategory::InvalidInput,
            invalid_input::PARAMETER,
            "identifiant de message invalide",
        ));
    }
    Ok(id.to_string())
}
