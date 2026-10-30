//! Parcours par le contrat : connexion prête, discussion, annulation, lecture du flux et du journal.

use mira_assembly::api::{
    AiRequest, AiResponse, ChatStart, ChatStream, Request, Response, SecretInput,
};
use mira_assembly::mira_vocabulary::{
    AiConnection, AiConnectionId, AiJournalEntry, AiProviderId, ChatMessage, ChatRequest, ChatRole,
    ContentKind, ContextPart, MiraError,
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
    chat_with(fixture, id, text, Vec::new()).await.unwrap()
}

/// Démarre une discussion d'un seul message portant ces parties de contexte, ou rend l'erreur
pub async fn chat_with(
    fixture: &Fixture,
    id: &AiConnectionId,
    text: &str,
    context: Vec<ContextPart>,
) -> Result<ChatStream, MiraError> {
    let message = ChatMessage::plain(ChatRole::User, text.to_string());
    let request = ChatRequest {
        model: None,
        system: None,
        messages: vec![message],
        max_output_tokens: None,
        temperature: None,
        tools: Vec::new(),
        context,
    };
    let start = ChatStart {
        connection: Some(id.clone()),
        request,
        allow_tools: false,
    };
    fixture.engine.chat_stream(start).await
}

/// Donne le consentement, teste la connexion (un relevé) puis lit les modèles depuis le cache
pub async fn consent_test_and_list_models(fixture: &Fixture, id: &AiConnectionId) {
    let consent = AiRequest::AcknowledgeAiConsent {
        connection: id.clone(),
        kinds: vec![ContentKind::MailMessage],
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

/// Lignes du journal des envois lues par le contrat, les plus récentes d'abord
pub async fn journal_rows(fixture: &Fixture) -> Vec<AiJournalEntry> {
    let request = AiRequest::ListAiJournal {
        limit: 50,
        before: None,
    };
    match ask_ai(fixture, request).await {
        AiResponse::Journal(rows) => rows,
        other => panic!("journal attendu : {other:?}"),
    }
}
