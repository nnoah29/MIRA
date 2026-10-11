//! Codes de la catégorie `network`.

use crate::error::code::ErrorCode;

/// Aucune connexion réseau.
pub const OFFLINE: ErrorCode = ErrorCode("network.offline");
/// Délai dépassé.
pub const TIMEOUT: ErrorCode = ErrorCode("network.timeout");
/// Serveur injoignable.
pub const UNREACHABLE: ErrorCode = ErrorCode("network.unreachable");
/// Connexion perdue en cours d'échange.
pub const CONNECTION_LOST: ErrorCode = ErrorCode("network.connection_lost");
