//! Ce que la tâche d'un compte possède : contexte, identifiants, budget, demandes, arrêt, état.

use std::sync::Arc;
use std::time::Duration;

use mira_vocabulary::AccountState;
use tokio::sync::{Notify, watch};

use super::config::SupervisorConfig;
use crate::sync::context::SyncContext;
use crate::sync::session::{ConnectionBudget, CredentialProvider};

/// Contexte d'exécution de la tâche de synchronisation d'un compte.
#[derive(Debug)]
pub struct AccountRuntime {
    pub ctx: SyncContext,
    pub credentials: Arc<dyn CredentialProvider>,
    pub budget: ConnectionBudget,
    pub requests: Arc<Notify>,
    pub shutdown: watch::Receiver<bool>,
    pub config: SupervisorConfig,
    pub state: AccountState,
}

/// Attend que l'arrêt soit demandé (ou que l'émetteur disparaisse).
pub async fn wait_shutdown(shutdown: &mut watch::Receiver<bool>) {
    loop {
        if *shutdown.borrow() {
            return;
        }
        if shutdown.changed().await.is_err() {
            return;
        }
    }
}

impl AccountRuntime {
    /// Attend le délai, ou une demande de synchronisation ; rend `false` si l'arrêt est demandé.
    pub async fn pause(&mut self, delay: Duration) -> bool {
        let requests = Arc::clone(&self.requests);
        tokio::select! {
            () = tokio::time::sleep(delay) => true,
            () = requests.notified() => true,
            () = wait_shutdown(&mut self.shutdown) => false,
        }
    }

    /// Attend une demande de l'utilisateur (reprise) ; rend `false` si l'arrêt est demandé.
    pub async fn wait_for_request(&mut self) -> bool {
        let requests = Arc::clone(&self.requests);
        tokio::select! {
            () = requests.notified() => true,
            () = wait_shutdown(&mut self.shutdown) => false,
        }
    }
}
