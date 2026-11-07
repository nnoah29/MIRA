//! Heure locale en heures pleines, derrière un trait pour que les tests fixent l'heure.

use chrono::Timelike;

/// Source de l'heure locale (0 à 23).
pub trait LocalClock: Send + Sync {
    /// Heure locale courante.
    fn hour(&self) -> u8;
}

/// Heure locale du système.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl LocalClock for SystemClock {
    /// Heure locale du système, de 0 à 23.
    fn hour(&self) -> u8 {
        u8::try_from(chrono::Local::now().hour()).unwrap_or(0)
    }
}
