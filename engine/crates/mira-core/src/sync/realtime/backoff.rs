//! Reconnexion avec délai croissant (10.6) : 1 s, 2 s, 4 s… plafonné ; réglable pour les tests.

use std::time::Duration;

/// Délais de reconnexion : premier délai et plafond.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackoffPolicy {
    pub initial: Duration,
    pub max: Duration,
}

impl Default for BackoffPolicy {
    /// 1 seconde au départ, 300 secondes au plus.
    fn default() -> Self {
        Self {
            initial: Duration::from_secs(1),
            max: Duration::from_secs(300),
        }
    }
}

/// Suite des délais d'attente entre deux tentatives de reconnexion.
#[derive(Debug, Clone, Copy)]
pub struct Backoff {
    policy: BackoffPolicy,
    next: Duration,
}

impl Backoff {
    /// Démarre une suite au premier délai de la politique.
    pub fn new(policy: BackoffPolicy) -> Self {
        Self {
            policy,
            next: policy.initial,
        }
    }

    /// Rend le délai à attendre maintenant et double le suivant, dans la limite du plafond.
    pub fn next_delay(&mut self) -> Duration {
        let current = self.next.min(self.policy.max);
        self.next = current.saturating_mul(2).min(self.policy.max);
        current
    }

    /// Revient au premier délai après une reconnexion réussie.
    pub fn reset(&mut self) {
        self.next = self.policy.initial;
    }
}
