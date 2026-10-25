//! Superviseur : une seule tâche de synchronisation par compte, arrêt propre borné (invariant 24, 10.11).

use std::sync::Arc;
use std::time::Duration;

use mira_vocabulary::AccountId;
use tokio::sync::watch;

use super::config::SupervisorConfig;
use super::running::{AccountTable, RunningAccount, stop_all};
use crate::sync::context::SyncContext;
use crate::sync::session::{ConnectionBudget, ConnectionBudgets, CredentialProvider};

/// Lance et arrête les tâches de synchronisation des comptes.
#[derive(Debug)]
pub struct SyncSupervisor {
    pub(super) config: SupervisorConfig,
    budgets: ConnectionBudgets,
    pub(super) shutdown: watch::Sender<bool>,
    pub(super) poll: watch::Sender<Option<Duration>>,
    pub(super) accounts: AccountTable,
}

impl SyncSupervisor {
    /// Crée un superviseur sans compte.
    pub fn new(config: SupervisorConfig) -> Self {
        Self::with_budgets(config, ConnectionBudgets::default())
    }

    /// Crée un superviseur dont les budgets de connexions sont partagés avec le reste du moteur.
    pub fn with_budgets(config: SupervisorConfig, budgets: ConnectionBudgets) -> Self {
        let (shutdown, _) = watch::channel(false);
        Self {
            config,
            budgets,
            shutdown,
            poll: watch::channel(config.poll_interval).0,
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
        let limit = ctx.provider.descriptor().limits.max_connections;
        let budget = self.budgets.budget_for(id.as_str(), limit);
        let running = RunningAccount::launch(
            ctx,
            credentials,
            self.config,
            budget,
            shutdown,
            self.poll.subscribe(),
        );
        table.insert(id, running);
        true
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
