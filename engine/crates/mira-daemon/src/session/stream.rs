//! État partagé d'un flux : fenêtre de crédits et signal d'arrêt, vus par la tâche et par la session.

use std::sync::Arc;

use tokio::sync::{Semaphore, watch};

use super::stop::Stop;

/// Crédits restants et arrêt demandé d'un flux ; se clone librement entre la session et la tâche du flux.
#[derive(Debug, Clone)]
pub struct Stream {
    credits: Arc<Semaphore>,
    stop: Arc<watch::Sender<Option<Stop>>>,
}

impl Stream {
    /// Flux neuf avec cette fenêtre de crédits.
    pub fn new(credit: u32) -> Self {
        Self {
            credits: Arc::new(Semaphore::new(credit as usize)),
            stop: Arc::new(watch::channel(None).0),
        }
    }

    /// Autorise `n` trames de plus.
    pub fn add_credit(&self, n: u32) {
        self.credits.add_permits(n as usize);
    }

    /// Demande l'arrêt ; la première raison l'emporte.
    pub fn stop(&self, why: Stop) {
        self.stop
            .send_if_modified(|current| current.replace(why).is_none());
    }

    /// Raison de l'arrêt demandé, s'il y en a une.
    pub fn reason(&self) -> Option<Stop> {
        *self.stop.borrow()
    }

    /// Consomme un crédit ; attend qu'il y en ait un, ou rend la raison de l'arrêt.
    pub async fn take_credit(&self) -> Result<(), Stop> {
        tokio::select! {
            permit = self.credits.acquire() => {
                permit.map(|permit| permit.forget()).map_err(|_| Stop::Cancelled)
            }
            why = self.stopped() => Err(why),
        }
    }

    /// Attend l'arrêt du flux et rend sa raison.
    pub async fn stopped(&self) -> Stop {
        let mut watcher = self.stop.subscribe();
        loop {
            if let Some(why) = *watcher.borrow_and_update() {
                return why;
            }
            if watcher.changed().await.is_err() {
                return Stop::Cancelled;
            }
        }
    }
}
