//! Drapeau d'annulation d'une opération, pour les travaux qui l'attendent sous cette forme.

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use super::handle::OperationHandle;

impl OperationHandle {
    /// Drapeau d'annulation partagé avec le registre
    pub fn cancel_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.cancel)
    }
}
