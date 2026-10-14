//! Superviseur : une seule tâche de synchronisation par compte, arrêt propre borné (invariant 24, 10.11).

use std::sync::Arc;
use std::time::Duration;

use mira_vocabulary::AccountId;
use tokio::sync::watch;

use super::config::SupervisorConfig;
use super::running::{AccountTable, RunningAccount, stop_all};
use crate::sync::context::SyncContext;
use crate::sync::session::{ConnectionBudget, CredentialProvider};

/// Lance et arrête les tâches de synchronisation des comptes.
#[derive(Debug)]
pub struct SyncSupervisor {
    config: SupervisorConfig,
    shutdown: watch::Sender<bool>,
    accounts: AccountTable,
}

impl SyncSupervisor {
    /// Crée un superviseur sans compte.
    pub fn new(config: SupervisorConfig) -> Self {
        let (shutdown, _) = watch::channel(false);
        Self {
            config,
            shutdown,
            accounts: AccountTable::default(),
        }
    }

    /// Démarre la tâche du compte ; rend `false` (sans rien lancer) si le compte a déjà sa tâche.
    pub fn start_account(
        &self,
        ctx: SyncContext,
        credentials: Arc<dyn CredentialProvider>,
    ) -> bool {
        let mut table = self.accounts.lock();
        let id = ctx.account.id.clone();
        if table.get(&id).is_some_and(RunningAccount::is_alive) {
            return false;
        }
        let shutdown = self.shutdown.subscribe();
        let running = RunningAccount::launch(ctx, credentials, self.config, shutdown);
        table.insert(id, running);
        true
    }

    /// Demande une synchronisation ; plusieurs demandes pendant une synchro n'en font qu'une.
    pub fn request_sync(&self, account: &AccountId) -> bool {
        match self.accounts.lock().get(account) {
            Some(run) => {
                run.requests.notify_one();
                true
            }
            None => false,
        }
    }

    /// Budget de connexions du compte, pour les tâches longues qui ouvrent leur propre connexion.
    pub fn budget_of(&self, account: &AccountId) -> Option<ConnectionBudget> {
        self.accounts
            .lock()
            .get(account)
            .map(|run| run.budget.clone())
    }

    /// Nombre de tâches de synchronisation encore actives.
    pub fn active_tasks(&self) -> usize {
        let table = self.accounts.lock();
        table.values().filter(|run| run.is_alive()).count()
    }

    /// Demande l'arrêt de toutes les tâches ; rend `false` si le délai est dépassé (tâches alors interrompues).
    pub async fn shutdown(&self, deadline: Duration) -> bool {
        let _ = self.shutdown.send(true);
        let running: Vec<RunningAccount> =
            self.accounts.lock().drain().map(|(_, run)| run).collect();
        stop_all(running, deadline).await
    }
}
