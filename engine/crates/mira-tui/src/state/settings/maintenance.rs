//! Onglet Maintenance : derniers volumes lus et dernier compte rendu.

use mira_assembly::storage::repositories::maintenance::StorageStats;

/// Volumes du miroir et compte rendu de la dernière opération de maintenance.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MaintenanceState {
    pub stats: Option<StorageStats>,
    pub report: Option<String>,
}
