//! Raccourcis des opérations qui modifient les conversations d'IA enregistrées : renommer, supprimer, tout effacer.

use mira_core::api::{AiRequest, AiResponse, Engine};
use mira_vocabulary::AiConversationMeta;

use super::call;

/// Renomme une conversation et rend ses métadonnées, ou le code d'erreur
pub async fn rename(engine: &Engine, id: &str, title: &str) -> Result<AiConversationMeta, String> {
    let request = AiRequest::RenameAiConversation {
        id: id.to_string(),
        title: title.to_string(),
    };
    match call(engine, request).await? {
        AiResponse::ConversationMeta(meta) => Ok(meta),
        other => panic!("réponse inattendue : {other:?}"),
    }
}

/// Supprime une conversation, ou rend le code d'erreur
pub async fn delete(engine: &Engine, id: &str) -> Result<(), String> {
    let request = AiRequest::DeleteAiConversation { id: id.to_string() };
    match call(engine, request).await? {
        AiResponse::Done => Ok(()),
        other => panic!("réponse inattendue : {other:?}"),
    }
}

/// Efface toutes les conversations, ou rend le code d'erreur
pub async fn clear(engine: &Engine) -> Result<(), String> {
    match call(engine, AiRequest::ClearAiConversations).await? {
        AiResponse::Done => Ok(()),
        other => panic!("réponse inattendue : {other:?}"),
    }
}
