//! État consultable d'une opération longue.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

use mira_vocabulary::OperationId;
use serde::{Deserialize, Serialize};

/// Avancement d'une opération longue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OperationStatus {
    Running { done: u64, total: u64 },
    Finished,
    Failed { code: String },
    Cancelled,
}

/// Fiche d'une opération : nature et avancement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationInfo {
    pub id: OperationId,
    pub kind: String,
    pub status: OperationStatus,
}

/// Table partagée des opérations connues du moteur.
pub(crate) type OperationTable = Arc<Mutex<HashMap<OperationId, OperationInfo>>>;

/// Accède à la table en ignorant un empoisonnement du verrou
pub(crate) fn lock_table(
    table: &OperationTable,
) -> MutexGuard<'_, HashMap<OperationId, OperationInfo>> {
    table
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
