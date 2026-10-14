//! Requêtes du domaine de la maintenance : volumes, caches, index, intégrité, exports.

use serde::{Deserialize, Serialize};

/// Opérations de maintenance du profil ; les exports n'écrivent jamais de secret.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum MaintenanceRequest {
    GetStorageStats,
    PurgeAttachmentCache,
    RebuildSearchIndex,
    CheckIntegrity,
    ExportDiagnostic { destination: String },
    ExportProfile { destination: String },
}
