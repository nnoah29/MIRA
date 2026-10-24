//! Ce que la tâche d'un compte possède : contexte, identifiants, budget, demandes, arrêt, état.

use std::sync::Arc;
use std::time::Duration;

use mira_vocabulary::{AccountState, ProviderLimits};
use tokio::sync::watch;

use super::config::{SupervisorConfig, provider_poll};
use crate::sync::context::SyncContext;
use crate::sync::report::ReportHandle;
use crate::sync::requests::SyncRequests;
use crate::sync::session::{ConnectionBudget, CredentialProvider};

/// Contexte d'exécution de la tâche de synchronisation d'un compte.
#[derive(Debug)]
pub struct AccountRuntime {
    pub ctx: SyncContext,
    pub credentials: Arc<dyn CredentialProvider>,
    pub budget: ConnectionBudget,
    pub requests: Arc<SyncRequests>,
    pub report: ReportHandle,
    pub shutdown: watch::Receiver<bool>,
    pub config: SupervisorConfig,
    pub poll: watch::Receiver<Option<Duration>>,
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
    /// Délai entre deux relevés : celui des réglages en vigueur, sinon le minimum du provider.
    pub fn poll_delay(&self, limits: &ProviderLimits) -> Duration {
        let chosen = *self.poll.borrow();
        chosen.unwrap_or_else(|| provider_poll(limits))
    }

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
