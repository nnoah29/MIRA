//! État partagé : file de comportements, requêtes reçues et déconnexions du client.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use super::behavior::Behavior;
use super::recorded::RecordedRequest;

/// Données mutables du faux serveur d'IA.
#[derive(Default)]
pub(super) struct AiState {
    queue: VecDeque<Behavior>,
    pub(super) requests: Vec<RecordedRequest>,
    pub(super) closed_by_client: usize,
}

/// Poignée clonable vers l'état partagé.
#[derive(Clone, Default)]
pub(super) struct AiShared {
    inner: Arc<Mutex<AiState>>,
}

impl AiShared {
    /// Exécute une action sur l'état en ignorant un verrou empoisonné
    pub(super) fn with<R>(&self, action: impl FnOnce(&mut AiState) -> R) -> R {
        let mut guard = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        action(&mut guard)
    }
}

impl AiState {
    /// Ajoute un comportement en fin de file
    pub(super) fn enqueue(&mut self, behavior: Behavior) {
        self.queue.push_back(behavior);
    }

    /// Prend le prochain comportement ; le dernier reste en file et se répète
    pub(super) fn next_behavior(&mut self) -> Behavior {
        if self.queue.len() > 1 {
            return self.queue.pop_front().unwrap_or_else(not_found);
        }
        self.queue.front().cloned().unwrap_or_else(not_found)
    }
}

/// Comportement par défaut quand rien n'est en file
fn not_found() -> Behavior {
    Behavior::Status {
        code: 404,
        headers: Vec::new(),
        body: r#"{"error":"no behavior queued"}"#.to_string(),
    }
}
