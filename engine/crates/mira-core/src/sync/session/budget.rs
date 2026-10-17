//! Budget de connexions d'un compte : jamais plus que la limite du provider (invariant 25).

use std::sync::Arc;
use std::sync::atomic::AtomicUsize;

use mira_vocabulary::error::codes::internal;
use mira_vocabulary::{ErrorCategory, MiraError};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

use super::waiting::WaitingSlot;

/// Permis de connexion partagés par toutes les connexions IMAP d'un compte.
#[derive(Debug, Clone)]
pub struct ConnectionBudget {
    semaphore: Arc<Semaphore>,
    limit: usize,
    waiting: Arc<AtomicUsize>,
}

impl ConnectionBudget {
    /// Crée un budget de `limit` connexions (au moins une).
    pub fn new(limit: u32) -> Self {
        let limit = usize::try_from(limit).unwrap_or(1).max(1);
        Self {
            semaphore: Arc::new(Semaphore::new(limit)),
            limit,
            waiting: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Nombre maximal de connexions simultanées.
    pub fn limit(&self) -> usize {
        self.limit
    }

    /// Nombre de permis encore libres.
    pub fn available(&self) -> usize {
        self.semaphore.available_permits()
    }

    /// Attend un permis de connexion.
    pub async fn acquire(&self) -> Result<OwnedSemaphorePermit, MiraError> {
        Arc::clone(&self.semaphore)
            .acquire_owned()
            .await
            .map_err(|_| MiraError::new(ErrorCategory::Internal, internal::BUG, "budget fermé"))
    }

    /// Prend un permis seulement s'il est disponible tout de suite.
    pub fn try_acquire(&self) -> Option<OwnedSemaphorePermit> {
        Arc::clone(&self.semaphore).try_acquire_owned().ok()
    }

    /// Attend un permis pour une demande explicite de l'utilisateur ; refuse si trop de demandes attendent déjà.
    pub async fn acquire_explicit(&self) -> Result<OwnedSemaphorePermit, MiraError> {
        let _slot = WaitingSlot::take(&self.waiting)?;
        self.acquire().await
    }
}
