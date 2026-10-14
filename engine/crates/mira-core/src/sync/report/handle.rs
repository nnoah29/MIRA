//! Poignée partagée du rapport d'un compte : la tâche l'écrit, la façade le lit.

use std::sync::{Arc, Mutex};

use super::state::SyncReport;

/// Rapport partagé entre la tâche d'un compte et ceux qui l'interrogent.
#[derive(Debug, Clone, Default)]
pub struct ReportHandle {
    state: Arc<Mutex<SyncReport>>,
}

impl ReportHandle {
    /// Copie du rapport courant.
    pub fn snapshot(&self) -> SyncReport {
        let mut copy = SyncReport::default();
        self.update(|state| copy = state.clone());
        copy
    }

    /// Modifie le rapport sous verrou, en ignorant un empoisonnement.
    pub(super) fn update(&self, change: impl FnOnce(&mut SyncReport)) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        change(&mut state);
    }
}
