//! Secret transmis une seule fois par le client (S2, P9) : jamais affiché, jamais renvoyé.

use serde::{Deserialize, Serialize};

use crate::ports::SecretValue;

/// Mot de passe ou mot de passe d'application saisi par l'utilisateur.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SecretInput(String);

impl SecretInput {
    /// Enveloppe une valeur saisie
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Dit si la valeur saisie est vide
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Convertit en valeur secrète du moteur, destinée au trousseau
    pub fn into_secret_value(self) -> SecretValue {
        SecretValue::new(self.0)
    }
}

impl std::fmt::Debug for SecretInput {
    /// Masque la valeur dans tout affichage de débogage
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("SecretInput(***)")
    }
}
