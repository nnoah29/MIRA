//! Codes de la catégorie `internal`.

use crate::error::code::ErrorCode;

/// Défaut interne du moteur.
pub const BUG: ErrorCode = ErrorCode("internal.bug");
