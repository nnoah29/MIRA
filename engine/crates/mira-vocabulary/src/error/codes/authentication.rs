//! Codes de la catégorie `authentication`.

use crate::error::code::ErrorCode;

/// Identifiants refusés par le serveur.
pub const REJECTED: ErrorCode = ErrorCode("authentication.rejected");
/// Jeton d'accès expiré.
pub const TOKEN_EXPIRED: ErrorCode = ErrorCode("authentication.token_expired");
/// Jeton révoqué par le service.
pub const TOKEN_REVOKED: ErrorCode = ErrorCode("authentication.token_revoked");
/// Le service exige un mot de passe d'application.
pub const APP_PASSWORD_REQUIRED: ErrorCode = ErrorCode("authentication.app_password_required");
/// Flux OAuth annulé par l'utilisateur.
pub const OAUTH_CANCELLED: ErrorCode = ErrorCode("authentication.oauth_cancelled");
/// Paramètre `state` OAuth inattendu.
pub const OAUTH_STATE_MISMATCH: ErrorCode = ErrorCode("authentication.oauth_state_mismatch");
/// Clé refusée par le service d'IA.
pub const AI_KEY_REJECTED: ErrorCode = ErrorCode("authentication.ai_key_rejected");
