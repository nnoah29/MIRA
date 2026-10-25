//! Réponse du domaine « réglages du moteur » : les réglages en vigueur.

use mira_vocabulary::EngineSettings;
use serde::{Deserialize, Serialize};

/// Réponse à une requête du domaine réglages du moteur.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum SettingsResponse {
    Settings(EngineSettings),
}
