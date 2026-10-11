//! Codes de la catégorie `local_storage`.

use crate::error::code::ErrorCode;

/// Disque plein.
pub const DISK_FULL: ErrorCode = ErrorCode("local_storage.disk_full");
/// Base locale corrompue.
pub const CORRUPTED: ErrorCode = ErrorCode("local_storage.corrupted");
/// Migration du schéma nécessaire.
pub const MIGRATION_REQUIRED: ErrorCode = ErrorCode("local_storage.migration_required");
/// Erreur d'entrée-sortie locale.
pub const IO: ErrorCode = ErrorCode("local_storage.io");
