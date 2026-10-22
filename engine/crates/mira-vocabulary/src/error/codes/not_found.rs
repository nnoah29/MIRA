//! Codes de la catégorie `not_found`.

use crate::error::code::ErrorCode;

/// Compte inconnu.
pub const ACCOUNT: ErrorCode = ErrorCode("not_found.account");
/// Message inconnu.
pub const MESSAGE: ErrorCode = ErrorCode("not_found.message");
/// Boîte inconnue.
pub const MAILBOX: ErrorCode = ErrorCode("not_found.mailbox");
/// Identité d'envoi inconnue.
pub const IDENTITY: ErrorCode = ErrorCode("not_found.identity");
/// Brouillon inconnu.
pub const DRAFT: ErrorCode = ErrorCode("not_found.draft");
/// Action en file inconnue.
pub const ACTION: ErrorCode = ErrorCode("not_found.action");
/// Opération longue inconnue.
pub const OPERATION: ErrorCode = ErrorCode("not_found.operation");
/// Provider inconnu du catalogue.
pub const PROVIDER: ErrorCode = ErrorCode("not_found.provider");
/// Pièce jointe inconnue.
pub const ATTACHMENT: ErrorCode = ErrorCode("not_found.attachment");
/// Fil inconnu.
pub const THREAD: ErrorCode = ErrorCode("not_found.thread");
/// Règle inconnue.
pub const RULE: ErrorCode = ErrorCode("not_found.rule");
/// Libellé local inconnu.
pub const LOCAL_LABEL: ErrorCode = ErrorCode("not_found.local_label");
