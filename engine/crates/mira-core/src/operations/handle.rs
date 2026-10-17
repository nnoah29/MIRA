//! Poignée d'une opération longue, tenue par la tâche qui l'exécute.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use mira_vocabulary::{EventKind, MiraError, OperationId};

use super::status::{OperationStatus, OperationTable, lock_table};
use crate::events::EventSink;

/// Permet à une tâche de publier sa progression et d'observer l'annulation.
#[derive(Debug, Clone)]
pub struct OperationHandle {
    id: OperationId,
    cancel: Arc<AtomicBool>,
    table: OperationTable,
    events: Arc<dyn EventSink>,
}

impl OperationHandle {
    /// Assemble une poignée (réservé au registre)
    pub(crate) fn new(
        id: OperationId,
        cancel: Arc<AtomicBool>,
        table: OperationTable,
        events: Arc<dyn EventSink>,
    ) -> Self {
        Self {
            id,
            cancel,
            table,
            events,
        }
    }

    /// Identifiant opaque de l'opération
    pub fn id(&self) -> &OperationId {
        &self.id
    }

    /// Indique si l'annulation a été demandée
    pub fn is_cancelled(&self) -> bool {
        self.cancel.load(Ordering::SeqCst)
    }

    /// Publie l'avancement (événement `OperationProgress`)
    pub fn progress(&self, done: u64, total: u64) {
        self.set_status(OperationStatus::Running { done, total });
        let _ = self.events.emit(EventKind::OperationProgress {
            operation: self.id.clone(),
            done,
            total,
        });
    }

    /// Termine l'opération : succès, échec ou annulation, avec l'événement correspondant
    pub fn complete(&self, outcome: Result<(), &MiraError>) {
        let operation = self.id.clone();
        let (status, kind) = match outcome {
            Ok(()) if self.is_cancelled() => (
                OperationStatus::Cancelled,
                EventKind::OperationCancelled { operation },
            ),
            Ok(()) => (
                OperationStatus::Finished,
                EventKind::OperationFinished { operation },
            ),
            Err(error) => (
                OperationStatus::Failed {
                    code: error.code.clone(),
                },
                EventKind::OperationFailed {
                    operation,
                    code: error.code.clone(),
                },
            ),
        };
        self.set_status(status);
        let _ = self.events.emit(kind);
    }

    /// Met à jour la fiche de l'opération
    fn set_status(&self, status: OperationStatus) {
        if let Some(info) = lock_table(&self.table).get_mut(&self.id) {
            info.status = status;
        }
    }
}
