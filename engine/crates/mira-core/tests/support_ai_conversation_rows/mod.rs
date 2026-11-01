//! Aides communes aux tests du dépôt des conversations d'IA.

use mira_core::storage::Database;
use mira_vocabulary::AiConversationMeta;

/// Enregistre une conversation de payload `{}` à cet instant et rend ses métadonnées
pub fn save(db: &Database, id: &str, title: &str, now: i64) -> AiConversationMeta {
    db.write(|tx| tx.save_ai_conversation(id, title, 1, "{}", now))
        .unwrap()
}

/// Identifiants listés, dans l'ordre rendu
pub fn listed_ids(db: &Database) -> Vec<String> {
    db.read(|tx| tx.list_ai_conversations(200, None))
        .unwrap()
        .into_iter()
        .map(|item| item.id)
        .collect()
}
