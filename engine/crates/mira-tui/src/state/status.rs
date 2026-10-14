//! Barre d'état : message à montrer, synchro en cours, file d'actions et connexion.

/// Contenu de la barre d'état.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StatusState {
    pub message: Option<String>,
    pub syncing: bool,
    pub pending_actions: u64,
    pub offline: bool,
    pub sync_error: Option<String>,
}
