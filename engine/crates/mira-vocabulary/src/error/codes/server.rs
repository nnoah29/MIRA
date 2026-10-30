//! Codes de la catégorie `server`.

use crate::error::code::ErrorCode;

/// Quota de la boîte dépassé.
pub const QUOTA_EXCEEDED: ErrorCode = ErrorCode("server.quota_exceeded");
/// Message trop gros pour le serveur.
pub const MESSAGE_TOO_LARGE: ErrorCode = ErrorCode("server.message_too_large");
/// Le serveur limite le débit des requêtes.
pub const RATE_LIMITED: ErrorCode = ErrorCode("server.rate_limited");
/// Boîte absente côté serveur.
pub const MAILBOX_MISSING: ErrorCode = ErrorCode("server.mailbox_missing");
/// Extension IMAP attendue absente.
pub const EXTENSION_MISSING: ErrorCode = ErrorCode("server.extension_missing");
/// Commande refusée par le serveur.
pub const REJECTED: ErrorCode = ErrorCode("server.rejected");
/// Réponse du serveur non conforme au protocole.
pub const PROTOCOL: ErrorCode = ErrorCode("server.protocol");
/// Le service d'IA limite le débit des requêtes.
pub const AI_RATE_LIMITED: ErrorCode = ErrorCode("server.ai_rate_limited");
/// Service d'IA indisponible.
pub const AI_UNAVAILABLE: ErrorCode = ErrorCode("server.ai_unavailable");
/// Réponse du service d'IA non conforme au format attendu.
pub const AI_BAD_RESPONSE: ErrorCode = ErrorCode("server.ai_bad_response");
