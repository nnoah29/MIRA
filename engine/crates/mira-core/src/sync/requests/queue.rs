//! File des demandes d'un compte : fusion des demandes identiques, réveil de la tâche (invariant 24).

use std::collections::BTreeSet;
use std::mem;
use std::sync::{Mutex, MutexGuard};

use mira_vocabulary::{MiraError, OperationId};
use tokio::sync::{Notify, oneshot};

use super::scope::RequestScope;
use super::taken::TakenRequests;

/// Accusé de fin d'une demande : la passe qui l'a servie a réussi ou échoué.
pub type Ticket = oneshot::Sender<Result<(), MiraError>>;

/// Demandes reçues et pas encore servies.
#[derive(Debug, Default)]
struct Pending {
    scopes: BTreeSet<RequestScope>,
    tickets: Vec<Ticket>,
    operation: Option<OperationId>,
}

/// Demandes en attente et signal de réveil de la tâche du compte.
#[derive(Debug, Default)]
pub struct SyncRequests {
    pending: Mutex<Pending>,
    wake: Notify,
}

impl SyncRequests {
    /// Ajoute une demande ; la première opération donnée est celle que portera la passe.
    pub fn push(
        &self,
        scope: RequestScope,
        ticket: Option<Ticket>,
        operation: Option<OperationId>,
    ) {
        let mut pending = self.lock();
        pending.scopes.insert(scope);
        pending.tickets.extend(ticket);
        if pending.operation.is_none() {
            pending.operation = operation;
        }
        drop(pending);
        self.wake.notify_one();
    }

    /// Retire toutes les demandes en attente pour les servir ensemble.
    pub fn take(&self) -> TakenRequests {
        let pending = mem::take(&mut *self.lock());
        TakenRequests {
            scopes: pending.scopes.into_iter().collect(),
            tickets: pending.tickets,
            operation: pending.operation,
        }
    }

    /// Répond par un échec à toutes les demandes en attente (la session vient de se rompre).
    pub fn fail_all(&self, error: &MiraError) {
        self.take().settle(&Err(error.clone()));
    }

    /// Attend qu'une demande arrive (un signal déjà reçu n'est pas perdu).
    pub async fn notified(&self) {
        self.wake.notified().await;
    }

    /// Prend le verrou des demandes en ignorant un empoisonnement.
    fn lock(&self) -> MutexGuard<'_, Pending> {
        self.pending
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
