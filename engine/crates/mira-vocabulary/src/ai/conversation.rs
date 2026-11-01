//! Conversation de l'assistant IA enregistrée localement ; le payload est un texte JSON opaque pour le moteur.

use serde::{Deserialize, Serialize};

use crate::time::Timestamp;

/// Ce que la liste des conversations montre : titre et dates, jamais le contenu.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiConversationMeta {
    pub id: String,
    pub title: String,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub payload_version: u32,
}

/// Une conversation complète : ses métadonnées et le texte JSON que l'app a écrit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiConversation {
    #[serde(flatten)]
    pub meta: AiConversationMeta,
    pub payload: String,
}
