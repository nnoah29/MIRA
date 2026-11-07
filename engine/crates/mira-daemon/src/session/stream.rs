//! État partagé d'un flux : fenêtre de crédits et signal d'arrêt, vus par la tâche et par la session.

use std::sync::Arc;

use tokio::sync::{Semaphore, watch};

use super::stop::Stop;

/// Crédits et arrêt demandé d'un flux ; se clone librement entre la session et la tâche du flux.
#[derive(Debug, Clone)]
pub struct Stream {
    pub(super) credits: Arc<Semaphore>,
    pub(super) stop: Arc<watch::Sender<Option<Stop>>>,
}

impl Stream {
    /// Fenêtre maximale de crédits d'un flux : très au-delà de tout besoin réel, très en deçà de la limite du sémaphore.
    pub const MAX_CREDIT: usize = 1 << 24;

    /// Flux neuf avec cette fenêtre de crédits (plafonnée).
    pub fn new(credit: u32) -> Self {
        Self {
            credits: Arc::new(Semaphore::new((credit as usize).min(Self::MAX_CREDIT))),
            stop: Arc::new(watch::channel(None).0),
        }
    }

    /// Autorise `n` trames de plus, sans dépasser [`Stream::MAX_CREDIT`].
    pub fn add_credit(&self, n: u32) {
        let room = Self::MAX_CREDIT.saturating_sub(self.credits.available_permits());
        self.credits.add_permits((n as usize).min(room));
    }

    /// Crédits encore disponibles.
    pub fn available(&self) -> usize {
        self.credits.available_permits()
    }

    /// Demande l'arrêt ; la première raison l'emporte.
    pub fn stop(&self, why: Stop) {
        self.stop.send_if_modified(|current| {
            if current.is_some() {
                return false;
            }
            *current = Some(why);
            true
        });
    }

    /// Raison de l'arrêt demandé, s'il y en a une.
    pub fn reason(&self) -> Option<Stop> {
        *self.stop.borrow()
    }
}
