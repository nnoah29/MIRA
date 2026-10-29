//! Parcours par le contrat : connexion prête, discussion, annulation, lecture du flux et du journal.

use mira_assembly::api::{
    AiRequest, AiResponse, ChatStart, ChatStream, Request, Response, SecretInput,
};
use mira_assembly::mira_vocabulary::{
    AiConnection, AiConnectionId, AiProviderId, ChatMessage, ChatRequest, ChatRole,
};

use super::fixture::{Fixture, KEY};

/// Exécute une requête du domaine Ai et rend sa réponse
pub async fn ask_ai(fixture: &Fixture, request: AiRequest) -> AiResponse {
    match fixture.engine.execute(Request::Ai(request)).await.unwrap() {
        Response::Ai(answer) => answer,
        other => panic!("réponse inattendue : {other:?}"),
    }
}

/// Crée une connexion du service compatible vers le faux serveur, avec clé et modèle par défaut
pub async fn connection_ready(fixture: &Fixture) -> AiConnection {
    let add = AiRequest::AddAiConnection {
        provider: AiProviderId("generic".to_string()),
        label: "Local".to_string(),
        base_url: Some(fixture.server.base_url()),
    };
    let AiResponse::Connection(created) = ask_ai(fixture, add).await else {
        panic!("connexion attendue");
    };
    let key = SecretInput::new(KEY);
    let set_key = AiRequest::SetAiApiKey {
        connection: created.id.clone(),
        key,
    };
    ask_ai(fixture, set_key).await;
    let update = AiRequest::UpdateAiConnection {
        connection: created.id.clone(),
        label: "Local".to_string(),
        base_url: created.base_url.clone(),
        default_model: Some("model-a".to_string()),
    };
    let AiResponse::Connection(ready) = ask_ai(fixture, update).await else {
        panic!("connexion attendue");
    };
    ready
}

/// Démarre une discussion d'un seul message sur cette connexion
pub async fn chat(fixture: &Fixture, id: &AiConnectionId, text: &str) -> ChatStream {
    let message = ChatMessage {
        role: ChatRole::User,
        text: text.to_string(),
    };
    let request = ChatRequest {
        model: None,
        system: None,
        messages: vec![message],
        max_output_tokens: None,
        temperature: None,
        tools: Vec::new(),
        context: Vec::new(),
    };
    let start = ChatStart {
        connection: Some(id.clone()),
        request,
    };
    fixture.engine.chat_stream(start).await.unwrap()
}

/// Donne le consentement, teste la connexion (un relevé) puis lit les modèles depuis le cache
pub async fn consent_test_and_list_models(fixture: &Fixture, id: &AiConnectionId) {
    let consent = AiRequest::AcknowledgeAiConsent {
        connection: id.clone(),
    };
    ask_ai(fixture, consent).await;
    let test = AiRequest::TestAiConnection {
        connection: id.clone(),
    };
    assert_eq!(ask_ai(fixture, test).await, AiResponse::Done);
    let list = AiRequest::ListAiModels {
        connection: id.clone(),
        refresh: false,
    };
    let AiResponse::Models(models) = ask_ai(fixture, list).await else {
        panic!("liste attendue");
    };
    assert_eq!(models.models[0].id, "model-a");
}
