//! Récepteur de test du notifieur : garde ce qu'on lui remet ; horloge locale fixe.

use std::sync::{Arc, Mutex};

use mira_host::notifier::{LocalClock, Notification, NotificationSink};

/// Récepteur de test : garde les notifications reçues, dans l'ordre.
#[derive(Debug, Default)]
pub struct CollectingSink {
    shown: Mutex<Vec<Notification>>,
}

impl CollectingSink {
    /// Notifications reçues jusqu'ici.
    pub fn shown(&self) -> Vec<Notification> {
        self.shown.lock().unwrap().clone()
    }
}

impl NotificationSink for CollectingSink {
    /// Garde la notification.
    fn show(&self, notification: Notification) {
        self.shown.lock().unwrap().push(notification);
    }
}

/// Heure locale fixe.
pub struct FixedClock(pub u8);

impl LocalClock for FixedClock {
    /// Toujours la même heure.
    fn hour(&self) -> u8 {
        self.0
    }
}

/// Récepteur et horloge prêts à passer au notifieur.
pub fn probes(hour: u8) -> (Arc<CollectingSink>, Arc<FixedClock>) {
    (
        Arc::new(CollectingSink::default()),
        Arc::new(FixedClock(hour)),
    )
}
