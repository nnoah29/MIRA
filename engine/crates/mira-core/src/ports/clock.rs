//! Horloge injectable : réelle pour la production, manuelle pour les tests.

use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use mira_vocabulary::Timestamp;

/// Source du temps du moteur.
pub trait Clock: Send + Sync + std::fmt::Debug {
    /// Donne l'instant courant
    fn now(&self) -> Timestamp;
}

/// Horloge réelle du système.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    /// Lit l'heure système en millisecondes Unix
    fn now(&self) -> Timestamp {
        let elapsed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| i64::try_from(duration.as_millis()).unwrap_or(i64::MAX))
            .unwrap_or(0);
        Timestamp::from_millis(elapsed)
    }
}

/// Horloge pilotée à la main, pour des tests déterministes.
#[derive(Debug)]
pub struct ManualClock {
    millis: AtomicI64,
}

impl ManualClock {
    /// Crée une horloge arrêtée à l'instant donné
    pub fn new(start: Timestamp) -> Self {
        Self {
            millis: AtomicI64::new(start.as_millis()),
        }
    }

    /// Replace l'horloge à un instant précis
    pub fn set(&self, instant: Timestamp) {
        self.millis.store(instant.as_millis(), Ordering::SeqCst);
    }

    /// Avance l'horloge de `delta` millisecondes
    pub fn advance_millis(&self, delta: i64) {
        self.millis.fetch_add(delta, Ordering::SeqCst);
    }
}

impl Clock for ManualClock {
    /// Donne l'instant réglé à la main
    fn now(&self) -> Timestamp {
        Timestamp::from_millis(self.millis.load(Ordering::SeqCst))
    }
}
