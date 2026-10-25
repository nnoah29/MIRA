//! Requêtes du domaine « réglages du moteur » : lecture et remplacement.

use mira_vocabulary::EngineSettings;
use serde::{Deserialize, Serialize};

/// Opérations sur les réglages que le moteur applique lui-même.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum SettingsRequest {
    GetSettings,
    SetSettings { settings: EngineSettings },
}
