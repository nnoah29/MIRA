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
