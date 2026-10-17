//! File d'attente bornée d'un budget : au-delà, une demande explicite est refusée au lieu de s'empiler.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use mira_vocabulary::error::codes::server;
use mira_vocabulary::{ErrorCategory, MiraError};

const MAX_WAITING: usize = 16;

/// Place dans la file d'attente d'un compte, rendue à la destruction (même si la tâche est interrompue).
#[derive(Debug)]
pub(super) struct WaitingSlot {
    counter: Arc<AtomicUsize>,
}

impl WaitingSlot {
    /// Prend une place d'attente, ou refuse si le compte a déjà trop de demandes en attente
    pub(super) fn take(counter: &Arc<AtomicUsize>) -> Result<Self, MiraError> {
        let queued = counter.fetch_add(1, Ordering::SeqCst);
        let slot = Self {
            counter: Arc::clone(counter),
        };
        if queued >= MAX_WAITING {
            return Err(MiraError::new(
                ErrorCategory::Server,
                server::RATE_LIMITED,
                "trop d'opérations réseau en attente pour ce compte",
            ));
        }
        Ok(slot)
    }
}

impl Drop for WaitingSlot {
    /// Rend la place d'attente
    fn drop(&mut self) {
        self.counter.fetch_sub(1, Ordering::SeqCst);
    }
}
