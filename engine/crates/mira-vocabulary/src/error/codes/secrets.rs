//! Codes de la catégorie `secrets`.

use crate::error::code::ErrorCode;

/// Trousseau de secrets indisponible.
pub const KEYRING_UNAVAILABLE: ErrorCode = ErrorCode("secrets.keyring_unavailable");
/// Trousseau verrouillé.
pub const KEYRING_LOCKED: ErrorCode = ErrorCode("secrets.keyring_locked");
/// Secret attendu absent du trousseau.
pub const SECRET_MISSING: ErrorCode = ErrorCode("secrets.secret_missing");
