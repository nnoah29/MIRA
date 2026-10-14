//! Commandes adressées aux tâches en cours : demandes de synchronisation, rapport, arrêt d'un compte.

use mira_vocabulary::{AccountId, OperationId};

use super::sync_supervisor::SyncSupervisor;
use crate::sync::report::SyncReport;
use crate::sync::requests::{RequestScope, Ticket};

impl SyncSupervisor {
    /// Demande une synchronisation complète ; plusieurs demandes pendant une synchro n'en font qu'une.
    pub fn request_sync(&self, account: &AccountId) -> bool {
        self.submit(account, RequestScope::Account, None, None)
    }

    /// Ajoute une demande à la file du compte ; rend `false` si le compte n'a pas de tâche.
    pub fn submit(
        &self,
        account: &AccountId,
        scope: RequestScope,
        ticket: Option<Ticket>,
        operation: Option<OperationId>,
    ) -> bool {
        match self.accounts.lock().get(account) {
            Some(run) => {
                run.requests.push(scope, ticket, operation);
                true
            }
            None => false,
        }
    }

    /// Rapport d'activité du compte, s'il a une tâche.
    pub fn report_of(&self, account: &AccountId) -> Option<SyncReport> {
        let table = self.accounts.lock();
        table.get(account).map(|run| run.report.snapshot())
    }

    /// Arrête la tâche d'un compte et attend sa fin ; rend `false` si le compte n'avait pas de tâche.
    pub async fn stop_account(&self, account: &AccountId) -> bool {
        let removed = self.accounts.lock().remove(account);
        let Some(run) = removed else {
            return false;
        };
        run.task.abort();
        let _ = run.task.await;
        true
    }

    /// Interrompt toutes les tâches sans attendre (moteur détruit ou fermé sans délai).
    pub fn abort_all(&self) {
        let _ = self.shutdown.send(true);
        for (_, run) in self.accounts.lock().drain() {
            run.task.abort();
        }
    }
}
