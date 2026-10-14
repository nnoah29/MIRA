//! État partagé des tâches de rejeu : configuration, base, événements, réveil et arrêt.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use tokio::sync::Notify;

use super::gates::AccountGates;
use crate::actions::QueueContext;
use crate::api::EngineConfig;
use crate::events::EventBus;
use crate::storage::Database;
use crate::sync::ConnectionBudgets;

/// Ce que partagent le superviseur et les tâches de rejeu d'un moteur.
#[derive(Debug)]
pub(crate) struct ReplayShared {
    pub(crate) config: EngineConfig,
    pub(crate) database: Arc<Database>,
    pub(crate) events: EventBus,
    pub(crate) budgets: ConnectionBudgets,
    pub(crate) gates: AccountGates,
    wake: Notify,
    stopping: AtomicBool,
}

impl ReplayShared {
    /// Prépare l'état partagé d'un moteur qui vient de s'ouvrir
    pub(crate) fn new(
        config: EngineConfig,
        database: Arc<Database>,
        events: EventBus,
        budgets: ConnectionBudgets,
    ) -> Self {
        Self {
            config,
            database,
            events,
            budgets,
            gates: AccountGates::default(),
            wake: Notify::new(),
            stopping: AtomicBool::new(false),
        }
    }

    /// Contexte des opérations locales de la file
    pub(crate) fn queue_context(&self) -> QueueContext<'_> {
        QueueContext {
            db: &self.database,
            clock: self.config.clock.as_ref(),
            ids: self.config.ids.as_ref(),
            events: &self.events,
        }
    }

    /// Dit si l'arrêt du moteur a été demandé
    pub(crate) fn is_stopping(&self) -> bool {
        self.stopping.load(Ordering::SeqCst)
    }

    /// Demande l'arrêt et réveille toutes les tâches
    pub(crate) fn request_stop(&self) {
        self.stopping.store(true, Ordering::SeqCst);
        self.wake.notify_waiters();
    }

    /// Réveille les tâches : du travail est peut-être prêt
    pub(crate) fn wake_all(&self) {
        self.wake.notify_waiters();
    }

    /// Attend un réveil ou, au plus, la durée donnée
    pub(crate) async fn wait(&self, patience: Duration) {
        tokio::select! {
            () = self.wake.notified() => {}
            () = tokio::time::sleep(patience) => {}
        }
    }
}
