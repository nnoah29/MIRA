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
/// Crédit ou quota du service d'IA épuisé.
pub const AI_QUOTA_EXHAUSTED: ErrorCode = ErrorCode("server.ai_quota_exhausted");
/// Le fichier téléchargé ne correspond pas à la somme de contrôle attendue.
pub const AI_LOCAL_CHECKSUM_MISMATCH: ErrorCode = ErrorCode("server.ai_local_checksum_mismatch");
/// Le serveur du modèle local s'est arrêté ou n'a pas pu démarrer.
pub const AI_LOCAL_RUNTIME_FAILED: ErrorCode = ErrorCode("server.ai.local_runtime_failed");
/// Le serveur du modèle local n'est pas prêt dans le délai imparti.
pub const AI_LOCAL_START_TIMEOUT: ErrorCode = ErrorCode("server.ai_local_start_timeout");
