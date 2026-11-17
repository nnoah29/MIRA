//! Style de rédaction de l'utilisateur : profil calculé sur ses mails envoyés, réglages et état montré à l'écran.

use serde::{Deserialize, Serialize};

use crate::time::Timestamp;

/// Nombre minimal de mails exploitables pour qu'un profil serve.
pub const STYLE_MIN_MAILS: u32 = 5;

/// Registre de politesse : tutoiement, vouvoiement, mélange des deux, ou aucune marque.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StyleRegister {
    Tu,
    Vous,
    Mixed,
    #[default]
    Unknown,
}

/// Ce que le moteur sait du style de l'utilisateur ; aucun nom, adresse ni extrait de mail.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StyleProfile {
    pub samples: u32,
    pub register: StyleRegister,
    pub greetings: Vec<String>,
    pub median_words: u32,
    pub median_sentence_words: u32,
    pub exclamation_permille: u32,
    pub emoji: bool,
}

impl StyleProfile {
    /// Dit si assez de mails ont servi pour que le profil soit utilisé.
    pub fn usable(&self) -> bool {
        self.samples >= STYLE_MIN_MAILS
    }
}

/// Réglages du style : désactivé par défaut ; les notes sont écrites par l'utilisateur.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StyleSettings {
    pub enabled: bool,
    pub notes: String,
}

/// Ce que montre l'écran du style : réglages, profil et date de la dernière analyse.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StyleStatus {
    pub settings: StyleSettings,
    pub profile: Option<StyleProfile>,
    pub analyzed_at: Option<Timestamp>,
}
