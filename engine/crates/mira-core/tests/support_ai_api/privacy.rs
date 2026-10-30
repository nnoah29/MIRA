//! Raccourcis des opérations de confidentialité : types autorisés, masquage, consentement par type, journal.

use mira_core::api::{AiRequest, AiResponse, Engine, Request};
use mira_vocabulary::{AiConnection, AiConnectionId, AiJournalEntry, ContentKind, MaskingSettings};

use super::calls::{call, call_connection};

/// Consentement donné pour ces types, ou rend le code d'erreur
pub async fn acknowledge(
    engine: &Engine,
    connection: &AiConnectionId,
    kinds: Vec<ContentKind>,
) -> Result<AiConnection, String> {
    let request = AiRequest::AcknowledgeAiConsent {
        connection: connection.clone(),
        kinds,
    };
    call_connection(engine, request).await
}

/// Consentement donné par un message JSON sans le champ `kinds`, comme l'écrit un client du contrat 1.1
pub async fn acknowledge_without_kinds(
    engine: &Engine,
    connection: &AiConnectionId,
) -> Result<AiConnection, String> {
    let json = serde_json::json!({
        "domain": "ai",
        "request": {"op": "acknowledge_ai_consent", "connection": connection.as_str()}
    });
    let Request::Ai(request) = serde_json::from_value(json).unwrap() else {
        panic!("requête du domaine Ai attendue");
    };
    call_connection(engine, request).await
}

/// Remplace la liste des types autorisés, ou rend le code d'erreur
pub async fn set_allowed(
    engine: &Engine,
    connection: &AiConnectionId,
    kinds: Vec<ContentKind>,
) -> Result<AiConnection, String> {
    let request = AiRequest::SetAiAllowedContent {
        connection: connection.clone(),
        kinds,
    };
    call_connection(engine, request).await
}

/// Remplace le réglage du masquage, ou rend le code d'erreur
pub async fn set_masking(
    engine: &Engine,
    connection: &AiConnectionId,
    masking: MaskingSettings,
) -> Result<AiConnection, String> {
    let request = AiRequest::SetAiMasking {
        connection: connection.clone(),
        masking,
    };
    call_connection(engine, request).await
}

/// Page du journal des envois, ou le code d'erreur
pub async fn journal(
    engine: &Engine,
    limit: u32,
    before: Option<&str>,
) -> Result<Vec<AiJournalEntry>, String> {
    let request = AiRequest::ListAiJournal {
        limit,
        before: before.map(str::to_string),
    };
    match call(engine, request).await? {
        AiResponse::Journal(rows) => Ok(rows),
        other => panic!("réponse inattendue : {other:?}"),
    }
}
