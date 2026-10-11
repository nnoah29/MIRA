//! État partagé entre le handle du serveur et ses sessions.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use super::mail::ReceivedMail;

/// Messages reçus et pannes armées par les tests.
#[derive(Debug, Default)]
pub(crate) struct SmtpState {
    pub(crate) received: Vec<ReceivedMail>,
    pub(crate) failures: VecDeque<String>,
    pub(crate) drop_after_data: bool,
}

/// Poignée clonable vers l'état partagé.
#[derive(Debug, Clone, Default)]
pub(crate) struct SmtpShared {
    inner: Arc<Mutex<SmtpState>>,
}

impl SmtpShared {
    /// Exécute une action sur l'état en ignorant un verrou empoisonné
    pub(crate) fn with<R>(&self, action: impl FnOnce(&mut SmtpState) -> R) -> R {
        let mut guard = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        action(&mut guard)
    }
}

impl SmtpState {
    /// Enregistre un message reçu et renvoie son identifiant de file
    pub(crate) fn record(&mut self, mail: ReceivedMail) -> String {
        self.received.push(mail);
        format!("mira{:06}", self.received.len())
    }
}
