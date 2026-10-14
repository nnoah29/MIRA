//! Oubli des opérations terminées les plus anciennes, pour que la table des opérations reste bornée.

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use mira_vocabulary::OperationId;

use super::status::{OperationStatus, OperationTable, lock_table};

const MAX_KEPT: usize = 256;

/// Drapeaux d'annulation des opérations connues.
pub(super) type Cancellations = Arc<Mutex<HashMap<OperationId, Arc<AtomicBool>>>>;

/// Identifiants des opérations dans l'ordre de leur création.
pub(super) type CreationOrder = Arc<Mutex<VecDeque<OperationId>>>;

/// Dit si l'opération est terminée (réussie, échouée ou annulée) ou déjà oubliée
fn is_finished(table: &OperationTable, id: &OperationId) -> bool {
    let guard = lock_table(table);
    let running = |status: &OperationStatus| matches!(status, OperationStatus::Running { .. });
    guard.get(id).is_none_or(|info| !running(&info.status))
}

/// Note l'opération qui commence puis oublie les plus anciennes opérations terminées au-delà de la borne
pub(super) fn remember_and_trim(
    table: &OperationTable,
    cancellations: &Cancellations,
    order: &CreationOrder,
    started: OperationId,
) {
    let mut queue = order.lock().unwrap_or_else(|p| p.into_inner());
    queue.push_back(started);
    let mut index = 0;
    while queue.len() > MAX_KEPT && index < queue.len() {
        if !is_finished(table, &queue[index]) {
            index += 1;
        } else if let Some(gone) = queue.remove(index) {
            lock_table(table).remove(&gone);
            let mut flags = cancellations.lock().unwrap_or_else(|p| p.into_inner());
            flags.remove(&gone);
        }
    }
}
