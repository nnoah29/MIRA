//! Compte des clients attachés après leur poignée de main, observable pour décider de l'arrêt à l'inactivité.

use std::sync::Arc;

use tokio::sync::watch;

/// Nombre de clients actuellement attachés ; les copies partagent le même compte.
#[derive(Debug, Clone)]
pub struct Clients {
    count: Arc<watch::Sender<usize>>,
}

/// Un client attaché ; le compte baisse quand ce garde disparaît.
#[derive(Debug)]
pub struct ClientGuard {
    count: Arc<watch::Sender<usize>>,
}

impl Default for Clients {
    /// Aucun client attaché.
    fn default() -> Self {
        Self {
            count: Arc::new(watch::channel(0).0),
        }
    }
}

impl Clients {
    /// Compte un client de plus jusqu'à la fin de vie du garde rendu.
    pub fn attach(&self) -> ClientGuard {
        self.count.send_modify(|count| *count += 1);
        ClientGuard {
            count: self.count.clone(),
        }
    }

    /// Récepteur du nombre de clients attachés.
    pub fn subscribe(&self) -> watch::Receiver<usize> {
        self.count.subscribe()
    }
}

impl Drop for ClientGuard {
    /// Décompte le client.
    fn drop(&mut self) {
        self.count
            .send_modify(|count| *count = count.saturating_sub(1));
    }
}
