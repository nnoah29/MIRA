//! Codes de la catégorie `cancelled`.

use crate::error::code::ErrorCode;

/// Annulé à la demande de l'utilisateur.
pub const BY_USER: ErrorCode = ErrorCode("cancelled.by_user");
