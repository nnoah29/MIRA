//! Mémoire de l'IA : souvenirs durables de l'utilisateur, leur genre, leur origine et les résultats de recherche.

use serde::{Deserialize, Serialize};

use crate::time::Timestamp;

/// Ce que raconte un souvenir : un fait, une préférence ou une action menée.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryKind {
    Fact,
    Preference,
    Action,
}

/// Qui a demandé l'enregistrement : l'utilisateur lui-même ou l'IA avec son accord.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemorySource {
    User,
    Ai,
}

/// Un souvenir sans son vecteur ; `model_id` nomme le modèle d'embeddings qui l'a indexé.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryRecord {
    pub id: String,
    pub text: String,
    pub kind: MemoryKind,
    pub source: MemorySource,
    pub created_at: Timestamp,
    pub model_id: Option<String>,
}

/// Un souvenir retrouvé par la recherche, avec sa similarité (de -1 à 1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryHit {
    pub record: MemoryRecord,
    pub score: f32,
}

/// Résultat de recherche pour l'API : le score en millièmes, entre -1000 et 1000, sans flottant dans le contrat.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryMatch {
    pub record: MemoryRecord,
    pub score_permille: i32,
}

impl From<MemoryHit> for MemoryMatch {
    /// Arrondit le score vers zéro en millièmes, borné à la plage de la similarité cosinus
    fn from(hit: MemoryHit) -> Self {
        let clamped = hit.score.clamp(-1.0, 1.0);
        Self {
            record: hit.record,
            score_permille: (clamped * 1000.0) as i32,
        }
    }
}
