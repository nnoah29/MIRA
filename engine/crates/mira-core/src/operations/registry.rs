//! Registre des opérations longues d'un moteur.

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use mira_vocabulary::error::codes::not_found;
use mira_vocabulary::{ErrorCategory, MiraError, OperationId};

use super::handle::OperationHandle;
use super::retention::{CreationOrder, remember_and_trim};
use super::status::{OperationInfo, OperationStatus, OperationTable, lock_table};
use crate::events::EventSink;
use crate::ports::IdGenerator;

/// Crée les opérations, les retrouve et transmet les demandes d'annulation.
#[derive(Debug, Clone)]
pub struct OperationRegistry {
    ids: Arc<dyn IdGenerator>,
    events: Arc<dyn EventSink>,
    table: OperationTable,
    cancellations: Arc<Mutex<HashMap<OperationId, Arc<AtomicBool>>>>,
    order: CreationOrder,
}

impl OperationRegistry {
    /// Crée un registre vide
    pub fn new(ids: Arc<dyn IdGenerator>, events: Arc<dyn EventSink>) -> Self {
        let table = Arc::new(Mutex::new(HashMap::new()));
        Self {
            ids,
            events,
            table,
            cancellations: Arc::new(Mutex::new(HashMap::new())),
            order: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    /// Démarre une opération et rend la poignée que la tâche utilisera
    pub fn start(&self, kind: &str) -> OperationHandle {
        let id = OperationId::from_raw(self.ids.next_id());
        let cancel = Arc::new(AtomicBool::new(false));
        let info = OperationInfo {
            id: id.clone(),
            kind: kind.to_string(),
            status: OperationStatus::Running { done: 0, total: 0 },
        };
        lock_table(&self.table).insert(id.clone(), info);
        self.cancellations
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .insert(id.clone(), Arc::clone(&cancel));
        remember_and_trim(&self.table, &self.cancellations, &self.order, id.clone());
        OperationHandle::new(
            id,
            cancel,
            Arc::clone(&self.table),
            Arc::clone(&self.events),
        )
    }

    /// Fiche d'une opération connue
    pub fn get(&self, id: &OperationId) -> Result<OperationInfo, MiraError> {
        lock_table(&self.table)
            .get(id)
            .cloned()
            .ok_or_else(unknown_operation)
    }

    /// Demande l'annulation d'une opération ; la tâche s'arrête au prochain point de contrôle
    pub fn cancel(&self, id: &OperationId) -> Result<(), MiraError> {
        let flags = self.cancellations.lock().unwrap_or_else(|p| p.into_inner());
        let flag = flags.get(id).ok_or_else(unknown_operation)?;
        flag.store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }
}

/// Erreur d'opération inconnue
fn unknown_operation() -> MiraError {
    MiraError::new(
        ErrorCategory::NotFound,
        not_found::OPERATION,
        "opération inconnue",
    )
}
