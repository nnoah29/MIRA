//! Sonde de test du garde de terminal : compte les entrées et les restaurations.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Compteurs partagés entre un garde de test et le test qui l'observe.
#[derive(Debug, Clone, Default)]
pub struct TerminalProbe {
    entered: Arc<AtomicUsize>,
    restored: Arc<AtomicUsize>,
}

impl TerminalProbe {
    /// Note une entrée en mode plein écran.
    pub fn record_enter(&self) {
        self.entered.fetch_add(1, Ordering::SeqCst);
    }

    /// Note une restauration du terminal.
    pub fn record_restore(&self) {
        self.restored.fetch_add(1, Ordering::SeqCst);
    }

    /// Indique si le plein écran a été activé.
    pub fn entered(&self) -> bool {
        self.entered.load(Ordering::SeqCst) > 0
    }

    /// Indique si le terminal a été restauré.
    pub fn restored(&self) -> bool {
        self.restore_count() > 0
    }

    /// Nombre de restaurations effectuées.
    pub fn restore_count(&self) -> usize {
        self.restored.load(Ordering::SeqCst)
    }
}
