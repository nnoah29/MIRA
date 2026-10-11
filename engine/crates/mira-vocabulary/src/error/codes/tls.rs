//! Codes de la catégorie `tls`.

use crate::error::code::ErrorCode;

/// Certificat du serveur invalide ou non approuvé.
pub const INVALID_CERTIFICATE: ErrorCode = ErrorCode("tls.invalid_certificate");
/// Échec de la négociation TLS.
pub const HANDSHAKE: ErrorCode = ErrorCode("tls.handshake");
/// Connexion en clair refusée hors boucle locale.
pub const PLAINTEXT_REFUSED: ErrorCode = ErrorCode("tls.plaintext_refused");
