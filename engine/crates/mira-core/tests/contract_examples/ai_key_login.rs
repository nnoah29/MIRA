//! Exemples du contrat pour la connexion d'IA par navigateur : trois requêtes, une réponse.

use mira_core::api::{AiRequest, AiResponse};
use mira_vocabulary::AiConnectionId;

/// Les trois requêtes de la connexion par navigateur
pub fn requests(id: &AiConnectionId) -> Vec<AiRequest> {
    vec![
        AiRequest::StartAiKeyLogin {
            connection: id.clone(),
        },
        AiRequest::CompleteAiKeyLogin {
            flow: "flow-1".to_string(),
        },
        AiRequest::CancelAiKeyLogin {
            flow: "flow-1".to_string(),
        },
    ]
}

/// Réponse au démarrage d'une connexion par navigateur
pub fn response() -> AiResponse {
    AiResponse::KeyLoginStarted {
        flow: "flow-1".to_string(),
        authorize_url:
            "https://auth.example.test/auth?callback_url=http%3A%2F%2F127.0.0.1%3A40000%2Fcallback"
                .to_string(),
    }
}
