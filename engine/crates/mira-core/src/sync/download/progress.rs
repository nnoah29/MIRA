//! Publication de l'avancement d'une synchronisation (événements `SyncProgress`).

use mira_vocabulary::{EventKind, SyncStep};

use crate::sync::session::SyncSession;

/// Émet l'avancement d'une étape de la descente.
pub fn report_progress(session: &SyncSession, step: SyncStep, done: u64, total: u64) {
    session.ctx.emit(EventKind::SyncProgress {
        account: session.ctx.account.id.clone(),
        operation: session.operation.clone(),
        step,
        done,
        total,
    });
}

/// Convertit un compte d'éléments en entier non signé de 64 bits.
pub fn count(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}
