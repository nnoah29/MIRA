//! Service de synchronisation d'un moteur : superviseur, identifiants de production, arrêt.

use std::sync::Arc;
use std::time::Duration;

use mira_vocabulary::error::codes::incompatible_state;
use mira_vocabulary::{ErrorCategory, MiraError};
use tokio::runtime::Handle;

use super::credentials::ProfileCredentials;
use crate::api::EngineConfig;
use crate::events::{EventBus, EventSink};
use crate::storage::Database;
use crate::sync::{BackgroundSync, CredentialProvider, SyncSupervisor};

/// Ce que le service possède quand la synchronisation en arrière-plan est active.
#[derive(Debug)]
pub(super) struct ActiveSync {
    pub(super) supervisor: SyncSupervisor,
    pub(super) credentials: Arc<dyn CredentialProvider>,
    pub(super) config: EngineConfig,
    pub(super) database: Arc<Database>,
    pub(super) events: Arc<dyn EventSink>,
}

/// Synchronisation en arrière-plan du moteur ; sans effet quand elle est désactivée.
#[derive(Debug)]
pub(crate) struct SyncService {
    pub(super) active: Option<ActiveSync>,
}

impl SyncService {
    /// Démarre le superviseur et la tâche de chaque compte actif, si la configuration l'autorise et qu'un runtime tourne
    pub(crate) fn start(
        config: &EngineConfig,
        database: &Arc<Database>,
        events: &EventBus,
    ) -> Self {
        let BackgroundSync::On(settings) = config.network.background_sync else {
            return Self { active: None };
        };
        if Handle::try_current().is_err() {
            return Self { active: None };
        }
        let credentials = ProfileCredentials::new(config.clone(), Arc::clone(database));
        let service = Self {
            active: Some(ActiveSync {
                supervisor: SyncSupervisor::new(settings),
                credentials: Arc::new(credentials),
                config: config.clone(),
                database: Arc::clone(database),
                events: Arc::new(events.clone()),
            }),
        };
        service.start_known_accounts();
        service
    }

    /// Le service est actif, ou l'erreur « synchronisation désactivée »
    pub(super) fn require_active(&self) -> Result<&ActiveSync, MiraError> {
        self.active.as_ref().ok_or_else(|| {
            MiraError::new(
                ErrorCategory::IncompatibleState,
                incompatible_state::SYNC_DISABLED,
                "la synchronisation en arrière-plan n'est pas active dans ce moteur",
            )
        })
    }

    /// Interrompt toutes les tâches sans attendre
    pub(crate) fn abort_all(&self) {
        if let Some(active) = &self.active {
            active.supervisor.abort_all();
        }
    }

    /// Arrête les tâches dans le délai donné ; rend `false` si certaines ont dû être interrompues
    pub(crate) async fn shutdown(&self, deadline: Duration) -> bool {
        match &self.active {
            Some(active) => active.supervisor.shutdown(deadline).await,
            None => true,
        }
    }
}

impl Drop for SyncService {
    /// Un moteur détruit n'a plus de tâche de synchronisation
    fn drop(&mut self) {
        self.abort_all();
    }
}
