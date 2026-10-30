//! Connexions de test insérées en base et requêtes de discussion minimales.

use mira_core::api::ChatStart;
use mira_vocabulary::{
    AiConnection, AiConnectionId, AiProviderId, ChatMessage, ChatRequest, ChatRole, MaskingSettings,
};

use crate::ai_models_support::ModelsFixture;

/// Insère une connexion `id` du fournisseur `test-provider` avec cette adresse et ce modèle par défaut
pub fn add_chat_connection(
    fixture: &ModelsFixture,
    id: &str,
    base_url: Option<String>,
    default_model: Option<&str>,
) -> AiConnection {
    let connection = AiConnection {
        id: AiConnectionId::from_raw(id),
        provider: AiProviderId("test-provider".to_string()),
        label: format!("Essai {id}"),
        base_url,
        default_model: default_model.map(str::to_string),
        is_default: false,
        has_key: false,
        consent_at: None,
        models: None,
        allowed_kinds: Vec::new(),
        masking: MaskingSettings::remote_default(),
    };
    fixture
        .db
        .write(|tx| tx.insert_ai_connection(&connection, 1))
        .unwrap();
    connection
}

/// Enregistre le consentement de l'utilisateur pour cette connexion
pub fn give_consent(fixture: &ModelsFixture, connection: &AiConnection) {
    fixture
        .db
        .write(|tx| tx.set_ai_consent(&connection.id, 1_700_000_000_000))
        .unwrap();
}

/// Fait de cette connexion la connexion par défaut
pub fn make_default(fixture: &ModelsFixture, connection: &AiConnection) {
    fixture
        .db
        .write(|tx| tx.set_default_ai_connection(&connection.id))
        .unwrap();
}

/// Requête d'un seul message de l'utilisateur, sans modèle imposé
pub fn user_request(text: &str) -> ChatRequest {
    ChatRequest {
        model: None,
        system: None,
        messages: vec![ChatMessage::plain(ChatRole::User, text.to_string())],
        max_output_tokens: None,
        temperature: None,
        tools: Vec::new(),
        context: Vec::new(),
    }
}

/// Démarrage d'une discussion sur cette connexion avec un seul message
pub fn chat_start(connection: &AiConnection, text: &str) -> ChatStart {
    ChatStart {
        connection: Some(connection.id.clone()),
        request: user_request(text),
    }
}
