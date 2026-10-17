//! Identifiant opaque typé : une chaîne que seul le cœur sait fabriquer (invariant 30).

use std::fmt;
use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

pub mod keys;
pub mod kinds;

/// Identifiant opaque d'une entité, typé par son marqueur `K` ; sérialisé comme une chaîne.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Id<K> {
    value: String,
    kind: PhantomData<K>,
}

impl<K> Id<K> {
    /// Enveloppe une chaîne déjà produite (le vocabulaire ne génère jamais d'identifiant).
    pub fn from_raw(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            kind: PhantomData,
        }
    }

    /// Donne la chaîne opaque sous-jacente.
    pub fn as_str(&self) -> &str {
        &self.value
    }

    /// Consomme l'identifiant et rend sa chaîne.
    pub fn into_string(self) -> String {
        self.value
    }
}

impl<K> fmt::Display for Id<K> {
    /// Affiche la chaîne opaque telle quelle.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.value)
    }
}
