//! Contrôle d'une clé d'API saisie : sans blanc ni caractère de contrôle, au plus 512 octets.

use mira_vocabulary::error::codes::invalid_input;
use mira_vocabulary::{ErrorCategory, MiraError};

use crate::api::contract::SecretInput;
use crate::ports::SecretValue;

const MAX_KEY_BYTES: usize = 512;

/// Retire les blancs de bord puis refuse une clé vide, trop longue ou contenant blanc ou contrôle
pub(super) fn validate_key(input: SecretInput) -> Result<SecretValue, MiraError> {
    let raw = input.into_secret_value();
    let trimmed = raw.expose().trim();
    let acceptable = !trimmed.is_empty()
        && trimmed.len() <= MAX_KEY_BYTES
        && !trimmed.chars().any(|c| c.is_whitespace() || c.is_control());
    if acceptable {
        return Ok(SecretValue::new(trimmed));
    }
    Err(MiraError::new(
        ErrorCategory::InvalidInput,
        invalid_input::AI_KEY_INVALID,
        "clé d'API refusée : vide, trop longue, ou contenant un blanc ou un caractère de contrôle",
    ))
}
