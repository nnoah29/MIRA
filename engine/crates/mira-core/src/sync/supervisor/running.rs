//! Tâches de compte en cours : lancement, table protégée, arrêt borné.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use mira_vocabulary::AccountId;
use tokio::sync::{Notify, watch};
use tokio::task::JoinHandle;

use super::account_task::run_account;
use super::config::SupervisorConfig;
use super::runtime::AccountRuntime;
use crate::sync::context::SyncContext;
use crate::sync::session::{ConnectionBudget, CredentialProvider};

/// Tâche en cours d'un compte : ses demandes de synchronisation, son budget et son exécution.
#[derive(Debug)]
pub struct RunningAccount {
    pub requests: Arc<Notify>,
    pub budget: ConnectionBudget,
    pub task: JoinHandle<()>,
}

/// Table des comptes suivis, protégée par un verrou.
#[derive(Debug, Default)]
pub struct AccountTable {
    accounts: Mutex<HashMap<AccountId, RunningAccount>>,
}

impl RunningAccount {
    /// Lance la tâche du compte avec son budget de connexions propre.
    pub fn launch(
        ctx: SyncContext,
        credentials: Arc<dyn CredentialProvider>,
        config: SupervisorConfig,
        shutdown: watch::Receiver<bool>,
    ) -> Self {
        let budget = ConnectionBudget::new(ctx.provider.descriptor().limits.max_connections);
        let requests = Arc::new(Notify::new());
        let state = ctx.account.state.clone();
        let runtime = AccountRuntime {
            ctx,
            credentials,
            budget: budget.clone(),
            requests: Arc::clone(&requests),
            shutdown,
            config,
            state,
        };
        let task = tokio::spawn(run_account(runtime));
        Self {
            requests,
            budget,
            task,
        }
    }

    /// La tâche tourne-t-elle encore ?
    pub fn is_alive(&self) -> bool {
        !self.task.is_finished()
    }
}

impl AccountTable {
    /// Prend le verrou de la table en ignorant un empoisonnement.
    pub fn lock(&self) -> MutexGuard<'_, HashMap<AccountId, RunningAccount>> {
        self.accounts
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

/// Attend la fin des tâches dans le délai, puis interrompt celles qui restent ; rend `true` si tout s'est arrêté seul.
pub async fn stop_all(mut running: Vec<RunningAccount>, deadline: Duration) -> bool {
    let drained = tokio::time::timeout(deadline, async {
        for run in &mut running {
            let _ = (&mut run.task).await;
        }
    })
    .await
    .is_ok();
    for run in &running {
        run.task.abort();
    }
    drained
}
