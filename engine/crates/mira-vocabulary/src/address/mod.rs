//! Adresse e-mail validée et normalisée (le domaine est mis en minuscules).

use serde::{Deserialize, Serialize};

use crate::error::category::ErrorCategory;
use crate::error::codes::invalid_input;
use crate::error::mira_error::MiraError;

pub mod entry;
pub mod rules;

/// Adresse e-mail valide ; sérialisée comme une chaîne.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct EmailAddress {
    value: String,
}

impl EmailAddress {
    /// Analyse et normalise une adresse ; refuse toute forme invalide (`invalid_input.address`).
    pub fn parse(input: &str) -> Result<Self, MiraError> {
        match rules::split_address(input.trim()) {
            Some((local, domain)) => Ok(Self {
                value: format!("{local}@{}", domain.to_lowercase()),
            }),
            None => Err(MiraError::new(
                ErrorCategory::InvalidInput,
                invalid_input::ADDRESS,
                "adresse e-mail invalide",
            )),
        }
    }

    /// Donne l'adresse complète normalisée.
    pub fn as_str(&self) -> &str {
        &self.value
    }

    /// Donne la partie locale, avant le `@`.
    pub fn local_part(&self) -> &str {
        self.value.split_once('@').map_or("", |(local, _)| local)
    }

    /// Donne le domaine en minuscules, après le `@`.
    pub fn domain(&self) -> &str {
        self.value.split_once('@').map_or("", |(_, domain)| domain)
    }
}

impl TryFrom<String> for EmailAddress {
    type Error = MiraError;

    /// Valide une chaîne désérialisée comme n'importe quelle saisie.
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(&value)
    }
}

impl From<EmailAddress> for String {
    /// Rend l'adresse normalisée sous forme de chaîne.
    fn from(address: EmailAddress) -> Self {
        address.value
    }
}
