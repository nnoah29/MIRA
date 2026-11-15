//! Garde-fous d'un texte de souvenir : non vide, borné, sans secret ni donnée bancaire.

use mira_vocabulary::error::codes::invalid_input::{AI_MEMORY_SENSITIVE, AI_MEMORY_TEXT};
use mira_vocabulary::{ErrorCategory, MiraError, SensitiveCategory};

use crate::ai::masking::detect;

pub(super) const MAX_CHARS: usize = 2_000;
const FORBIDDEN: [SensitiveCategory; 4] = [
    SensitiveCategory::Secret,
    SensitiveCategory::Card,
    SensitiveCategory::Iban,
    SensitiveCategory::Otp,
];

/// Rend le texte nettoyé de ses blancs de bord, ou refuse un texte vide, trop long ou sensible
pub(super) fn checked_text(text: &str) -> Result<String, MiraError> {
    let trimmed = text.trim();
    if trimmed.is_empty() || trimmed.chars().count() > MAX_CHARS {
        return Err(MiraError::new(
            ErrorCategory::InvalidInput,
            AI_MEMORY_TEXT,
            "le souvenir doit contenir entre 1 et 2 000 caractères",
        ));
    }
    if !detect(trimmed, &FORBIDDEN).is_empty() {
        return Err(MiraError::new(
            ErrorCategory::InvalidInput,
            AI_MEMORY_SENSITIVE,
            "le souvenir contient une donnée sensible et n'est pas enregistré",
        ));
    }
    Ok(trimmed.to_string())
}
