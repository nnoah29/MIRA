//! Délai de relevé périodique : modifiable en cours de route, suivi par chaque tâche de compte.

use std::time::Duration;

use super::sync_supervisor::SyncSupervisor;

impl SyncSupervisor {
    /// Change le délai de relevé de toutes les tâches ; sans valeur, revient à celui de la configuration.
    pub fn set_poll(&self, poll: Option<Duration>) {
        self.poll.send_replace(poll.or(self.config.poll_interval));
    }
}
