//! Réglages du moteur appliqués aux tâches de synchronisation, au lancement puis à chaque changement.

use std::time::Duration;

use mira_vocabulary::EngineSettings;

use super::service::SyncService;

impl SyncService {
    /// Applique les réglages aux tâches en cours et à celles qui démarreront
    pub(crate) fn apply_settings(&self, settings: &EngineSettings) {
        let Some(active) = &self.active else {
            return;
        };
        let poll = settings
            .poll_secs
            .map(|secs| Duration::from_secs(u64::from(secs)));
        active.supervisor.set_poll(poll);
    }

    /// Applique les réglages enregistrés avant de lancer les tâches des comptes
    pub(super) fn apply_stored_settings(&self) {
        let Some(active) = &self.active else {
            return;
        };
        if let Ok(stored) = active.database.read(|tx| tx.get_engine_settings()) {
            self.apply_settings(&stored);
        }
    }
}
