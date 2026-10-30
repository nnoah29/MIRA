//! Réglages de masquage des données sensibles d'une connexion d'IA.

use serde::{Deserialize, Serialize};

/// Famille de données sensibles que le masquage sait reconnaître.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SensitiveCategory {
    Iban,
    Card,
    Secret,
    Otp,
    Email,
    Phone,
}

/// Le masquage est soit coupé, soit actif.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaskingMode {
    Off,
    Mask,
}

/// Mode et catégories masquées d'une connexion ; une aide au mieux, jamais une garantie.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaskingSettings {
    pub mode: MaskingMode,
    pub categories: Vec<SensitiveCategory>,
}

impl MaskingSettings {
    /// Catégories actives par défaut : celles dont l'oubli coûte cher, pas l'e-mail ni le téléphone.
    fn default_categories() -> Vec<SensitiveCategory> {
        vec![
            SensitiveCategory::Iban,
            SensitiveCategory::Card,
            SensitiveCategory::Secret,
            SensitiveCategory::Otp,
        ]
    }

    /// Réglage d'une connexion distante : masquage actif sur les catégories par défaut.
    pub fn remote_default() -> Self {
        Self {
            mode: MaskingMode::Mask,
            categories: Self::default_categories(),
        }
    }

    /// Réglage d'une connexion en boucle locale : masquage coupé, mêmes catégories.
    pub fn local_default() -> Self {
        Self {
            mode: MaskingMode::Off,
            categories: Self::default_categories(),
        }
    }
}

impl Default for MaskingSettings {
    /// Le réglage d'une connexion distante, le plus prudent.
    fn default() -> Self {
        Self::remote_default()
    }
}
