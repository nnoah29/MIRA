//! Emplacement de l'installation en cours : une seule à la fois, lisible par l'état.

use std::sync::{Arc, Mutex, PoisonError};

use mira_vocabulary::LocalInstallInfo;

/// Installation en cours partagée entre le moteur et sa tâche de fond.
#[derive(Debug, Clone, Default)]
pub struct LocalInstallSlot(Arc<Mutex<Option<LocalInstallInfo>>>);

impl LocalInstallSlot {
    /// Installation en cours, s'il y en a une
    pub fn current(&self) -> Option<LocalInstallInfo> {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// Réserve l'emplacement ; rend faux si une installation est déjà en cours
    pub fn try_begin(&self, info: LocalInstallInfo) -> bool {
        let mut slot = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        if slot.is_some() {
            return false;
        }
        *slot = Some(info);
        true
    }

    /// Libère l'emplacement
    pub fn end(&self) {
        *self.0.lock().unwrap_or_else(PoisonError::into_inner) = None;
    }
}
