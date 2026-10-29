//! Exemples du domaine « assistant d'IA » : connexions, clés, modèles.

use mira_core::api::{AiRequest, AiResponse, Request, Response, SecretInput};
use mira_vocabulary::{AiConnectionId, AiProviderId};

use super::ai_samples::{sample_connection, sample_models, sample_provider};
use super::{Named, numbered};

/// Les opérations de gestion : fournisseurs, création, mise à jour, suppression
fn management_requests(id: &AiConnectionId) -> Vec<AiRequest> {
    vec![
        AiRequest::ListAiProviders,
        AiRequest::ListAiConnections,
        AiRequest::AddAiConnection {
            provider: AiProviderId("generic".to_string()),
            label: "Mon service".to_string(),
            base_url: Some("https://ia.example.org/v1".to_string()),
        },
        AiRequest::UpdateAiConnection {
            connection: id.clone(),
            label: "Mon service".to_string(),
            base_url: None,
            default_model: Some("model-a".to_string()),
        },
        AiRequest::RemoveAiConnection {
            connection: id.clone(),
        },
        AiRequest::SetDefaultAiConnection {
            connection: id.clone(),
        },
    ]
}

/// Une requête par opération de `AiRequest`
pub fn requests() -> Vec<Named<Request>> {
    let id = AiConnectionId::from_raw("ai-1");
    let mut items = management_requests(&id);
    items.extend([
        AiRequest::SetAiApiKey {
            connection: id.clone(),
            key: SecretInput::new("sk-exemple"),
        },
        AiRequest::ClearAiApiKey {
            connection: id.clone(),
        },
        AiRequest::TestAiConnection {
            connection: id.clone(),
        },
        AiRequest::ListAiModels {
            connection: id.clone(),
            refresh: true,
        },
        AiRequest::AcknowledgeAiConsent { connection: id },
    ]);
    let ops: Vec<&str> = items.iter().map(op_name).collect();
    numbered("ai", &ops)
        .into_iter()
        .zip(items.into_iter().map(Request::Ai))
        .collect()
}

/// Une réponse par forme de `AiResponse`
pub fn responses() -> Vec<Named<Response>> {
    let items = [
        (
            "ai.providers",
            AiResponse::Providers(vec![sample_provider()]),
        ),
        (
            "ai.connections",
            AiResponse::Connections(vec![sample_connection()]),
        ),
        ("ai.connection", AiResponse::Connection(sample_connection())),
        ("ai.models", AiResponse::Models(sample_models())),
        ("ai.done", AiResponse::Done),
    ];
    let wrap = |(name, inner): (&str, AiResponse)| (name.to_string(), Response::Ai(inner));
    items.into_iter().map(wrap).collect()
}

/// Nom de l'opération, par `match` exhaustif
pub fn op_name(request: &AiRequest) -> &'static str {
    match request {
        AiRequest::ListAiProviders => "list_ai_providers",
        AiRequest::ListAiConnections => "list_ai_connections",
        AiRequest::AddAiConnection { .. } => "add_ai_connection",
        AiRequest::UpdateAiConnection { .. } => "update_ai_connection",
        AiRequest::RemoveAiConnection { .. } => "remove_ai_connection",
        AiRequest::SetDefaultAiConnection { .. } => "set_default_ai_connection",
        AiRequest::SetAiApiKey { .. } => "set_ai_api_key",
        AiRequest::ClearAiApiKey { .. } => "clear_ai_api_key",
        AiRequest::TestAiConnection { .. } => "test_ai_connection",
        AiRequest::ListAiModels { .. } => "list_ai_models",
        AiRequest::AcknowledgeAiConsent { .. } => "acknowledge_ai_consent",
    }
}
