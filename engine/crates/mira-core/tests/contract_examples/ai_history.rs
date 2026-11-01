//! Exemples du domaine « assistant d'IA » pour l'historique des conversations : requêtes, réponses et valeurs.

use mira_core::api::{AiRequest, AiResponse, Response};
use mira_vocabulary::{AiConversation, AiConversationMeta, Timestamp};

use super::Named;

/// Métadonnées d'exemple : une conversation créée puis mise à jour
pub fn sample_conversation_meta() -> AiConversationMeta {
    AiConversationMeta {
        id: "conversation-1".to_string(),
        title: "Où en est la facture ?".to_string(),
        created_at: Timestamp::from_millis(1_700_000_000_000),
        updated_at: Timestamp::from_millis(1_700_000_060_000),
        payload_version: 1,
    }
}

/// Conversation d'exemple avec un contenu minimal
pub fn sample_conversation() -> AiConversation {
    AiConversation {
        meta: sample_conversation_meta(),
        payload: "{\"messages\":[]}".to_string(),
    }
}

/// Une requête par opération de l'historique
pub fn requests() -> Vec<AiRequest> {
    vec![
        AiRequest::SaveAiConversation {
            id: "conversation-1".to_string(),
            title: "Où en est la facture ?".to_string(),
            payload_version: 1,
            payload: "{\"messages\":[]}".to_string(),
        },
        AiRequest::ListAiConversations {
            limit: 50,
            before: Some("conversation-1".to_string()),
        },
        AiRequest::GetAiConversation {
            id: "conversation-1".to_string(),
        },
    ]
}

/// Une réponse par forme de l'historique
pub fn responses() -> Vec<Named<Response>> {
    let items = [
        (
            "ai.conversations",
            AiResponse::Conversations(vec![sample_conversation_meta()]),
        ),
        (
            "ai.conversation",
            AiResponse::Conversation(sample_conversation()),
        ),
        (
            "ai.conversation_meta",
            AiResponse::ConversationMeta(sample_conversation_meta()),
        ),
    ];
    let wrap = |(name, inner): (&str, AiResponse)| (name.to_string(), Response::Ai(inner));
    items.into_iter().map(wrap).collect()
}
