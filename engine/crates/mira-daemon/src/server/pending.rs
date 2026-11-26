//! Connexions qui attendent leur `hello`, de la plus ancienne à la plus récente : à saturation, la plus ancienne cède sa place.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

use tokio::sync::Notify;

type Queue = Mutex<VecDeque<(u64, Arc<Notify>)>>;

/// Registre partagé des connexions en attente de leur `hello`.
#[derive(Debug, Clone, Default)]
pub struct PendingHellos {
    queue: Arc<Queue>,
    next: Arc<AtomicU64>,
}

/// Une connexion en attente ; elle sort du registre quand ce garde disparaît.
#[derive(Debug)]
pub struct PendingEntry {
    id: u64,
    evicted: Arc<Notify>,
    queue: Arc<Queue>,
}

impl PendingHellos {
    /// Inscrit une nouvelle connexion en attente, la plus récente de la file.
    pub fn register(&self) -> PendingEntry {
        let id = self.next.fetch_add(1, Ordering::Relaxed);
        let evicted = Arc::new(Notify::new());
        let mut queue = self.queue.lock().unwrap_or_else(PoisonError::into_inner);
        queue.push_back((id, evicted.clone()));
        PendingEntry {
            id,
            evicted,
            queue: self.queue.clone(),
        }
    }

    /// Nombre de connexions inscrites depuis le démarrage, évincées ou non ; sert à synchroniser les tests sur l'ordre d'arrivée.
    pub fn registered(&self) -> u64 {
        self.next.load(Ordering::Relaxed)
    }

    /// Prévient la plus ancienne connexion en attente qu'elle doit partir ; dit s'il y en avait une.
    pub fn evict_oldest(&self) -> bool {
        let mut queue = self.queue.lock().unwrap_or_else(PoisonError::into_inner);
        match queue.pop_front() {
            Some((_, evicted)) => {
                evicted.notify_one();
                true
            }
            None => false,
        }
    }
}

impl PendingEntry {
    /// Se termine quand cette connexion est évincée au profit d'une plus récente.
    pub async fn evicted(&self) {
        self.evicted.notified().await;
    }
}

impl Drop for PendingEntry {
    /// Retire la connexion du registre si elle n'en a pas déjà été évincée.
    fn drop(&mut self) {
        let mut queue = self.queue.lock().unwrap_or_else(PoisonError::into_inner);
        queue.retain(|(id, _)| *id != self.id);
    }
}
