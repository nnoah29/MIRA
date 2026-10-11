//! Instant en millisecondes depuis l'époque Unix, en UTC.

use serde::{Deserialize, Serialize};

/// Instant UTC ; sérialisé comme un entier de millisecondes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Timestamp {
    pub millis: i64,
}

impl Timestamp {
    /// Construit un instant à partir de millisecondes Unix.
    pub fn from_millis(millis: i64) -> Self {
        Self { millis }
    }

    /// Donne les millisecondes Unix.
    pub fn as_millis(&self) -> i64 {
        self.millis
    }

    /// Rend l'instant décalé de `delta` millisecondes (négatif pour reculer).
    pub fn plus_millis(&self, delta: i64) -> Self {
        Self {
            millis: self.millis.saturating_add(delta),
        }
    }
}
