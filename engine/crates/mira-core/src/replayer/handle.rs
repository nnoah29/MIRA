//! Poignée du rejeu en tâche de fond : démarrage, réveil, arrêt.

use std::sync::Arc;
use std::time::Duration;

use mira_vocabulary::AccountId;
use tokio::runtime::Handle;
use tokio::task::JoinHandle;

use super::shared::ReplayShared;
use super::supervisor::supervise;
use crate::api::EngineConfig;
use crate::events::EventBus;
use crate::storage::Database;
use crate::sync::ConnectionBudgets;

const STOP_PATIENCE: Duration = Duration::from_secs(30);

/// Le rejeu d'un moteur : le superviseur n'existe que si un runtime tokio est actif à l'ouverture.
#[derive(Debug)]
pub(crate) struct ReplayerHandle {
    shared: Arc<ReplayShared>,
    supervisor: Option<JoinHandle<()>>,
}

impl ReplayerHandle {
    /// Lance le superviseur dans le runtime courant, s'il y en a un
    pub(crate) fn start(
        config: EngineConfig,
        database: Arc<Database>,
        events: EventBus,
        budgets: ConnectionBudgets,
    ) -> Self {
        let shared = Arc::new(ReplayShared::new(config, database, events, budgets));
        let supervisor = Handle::try_current()
            .ok()
            .map(|runtime| runtime.spawn(supervise(Arc::clone(&shared))));
        Self { shared, supervisor }
    }

    /// Signale que du travail est peut-être prêt (soumission, relance, reprise)
    pub(crate) fn wake(&self) {
        self.shared.wake_all();
    }

    /// Attend la fin de la passe en cours du compte (au plus 30 s) ; le compte doit être marqué « en suppression » pour qu'aucune autre ne commence
    pub(crate) async fn stop_account(&self, id: &AccountId) {
        let gate = self.shared.gates.gate(id);
        let _ = tokio::time::timeout(STOP_PATIENCE, gate.lock()).await;
    }

    /// Arrête le superviseur et, avec lui, toutes les tâches de compte
    pub(crate) fn stop(&self) {
        self.shared.request_stop();
        if let Some(supervisor) = &self.supervisor {
            supervisor.abort();
        }
    }
}

impl Drop for ReplayerHandle {
    /// Un moteur détruit n'a plus de tâche de fond
    fn drop(&mut self) {
        self.stop();
    }
}
