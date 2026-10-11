//! Valeur de secret : jamais affichée, jamais sérialisée.

use std::fmt;

/// Secret en clair ; `Debug` et `Display` affichent toujours `***`.
#[derive(Clone)]
pub struct SecretValue(String);

impl SecretValue {
    /// Enveloppe une valeur secrète
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Donne la valeur en clair, pour l'envoyer au serveur seulement
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for SecretValue {
    /// Affiche `***` à la place de la valeur
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("***")
    }
}

impl fmt::Display for SecretValue {
    /// Affiche `***` à la place de la valeur
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("***")
    }
}
