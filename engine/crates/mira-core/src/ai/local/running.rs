//! Serveur en marche : processus, adresse, clé et dernier usage.

use std::time::{Duration, Instant};

use tokio::process::Child;

use super::runtime::LocalRuntimeHandle;
use crate::ports::SecretValue;

/// Serveur en marche : son processus, son adresse, sa clé et son dernier usage.
pub(super) struct Running {
    pub(super) model_id: String,
    pub(super) child: Child,
    root_url: String,
    key: String,
    last_used: Instant,
}

impl Running {
    /// Enveloppe un processus qui vient de répondre prêt
    pub(super) fn new(model_id: String, child: Child, root_url: String, key: String) -> Self {
        Self {
            model_id,
            child,
            root_url,
            key,
            last_used: Instant::now(),
        }
    }

    /// Adresses et clé à remettre à l'appelant
    pub(super) fn handle(&self) -> LocalRuntimeHandle {
        LocalRuntimeHandle {
            root_url: self.root_url.clone(),
            base_url: format!("{}/v1", self.root_url),
            api_key: SecretValue::new(self.key.clone()),
        }
    }

    /// Note qu'on vient de s'en servir
    pub(super) fn touch(&mut self) {
        self.last_used = Instant::now();
    }

    /// Temps écoulé depuis le dernier usage
    pub(super) fn idle_for(&self) -> Duration {
        self.last_used.elapsed()
    }
}
