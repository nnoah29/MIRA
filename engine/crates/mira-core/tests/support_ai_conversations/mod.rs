//! Moteur de test et raccourcis des opérations de conversation d'IA, qui rendent le code d'erreur en cas de refus.

#![allow(dead_code)]

pub mod changes;
pub mod history;

use std::sync::Arc;

use mira_core::api::{AiRequest, AiResponse, Engine, Request, Response};
use mira_core::ports::ManualClock;
use mira_vocabulary::{AiConversation, AiConversationMeta, CONTRACT_VERSION, Timestamp};

use crate::support_api;

/// Moteur ouvert sur un dossier temporaire, avec son horloge réglable
pub struct Fixture {
    pub engine: Engine,
    pub clock: Arc<ManualClock>,
    _dir: tempfile::TempDir,
}

/// Ouvre un moteur dont l'horloge part de 1 000
pub fn start() -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let clock = Arc::new(ManualClock::new(Timestamp::from_millis(1_000)));
    let mut config = support_api::config(dir.path());
    config.clock = clock.clone();
    let engine = Engine::open(config, CONTRACT_VERSION).unwrap();
    Fixture {
        engine,
        clock,
        _dir: dir,
    }
}

/// Exécute une requête du domaine et rend sa réponse, ou le code d'erreur
pub async fn call(engine: &Engine, request: AiRequest) -> Result<AiResponse, String> {
    match engine.execute(Request::Ai(request)).await {
        Ok(Response::Ai(answer)) => Ok(answer),
        Ok(other) => panic!("réponse inattendue : {other:?}"),
        Err(error) => Err(error.code),
    }
}

/// Enregistre une conversation (version de payload 1) et rend ses métadonnées
pub async fn save(
    engine: &Engine,
    id: &str,
    title: &str,
    payload: &str,
) -> Result<AiConversationMeta, String> {
    let request = AiRequest::SaveAiConversation {
        id: id.to_string(),
        title: title.to_string(),
        payload_version: 1,
        payload: payload.to_string(),
    };
    match call(engine, request).await? {
        AiResponse::ConversationMeta(meta) => Ok(meta),
        other => panic!("réponse inattendue : {other:?}"),
    }
}

/// Liste les conversations, ou rend le code d'erreur
pub async fn list(
    engine: &Engine,
    limit: u32,
    before: Option<&str>,
) -> Result<Vec<AiConversationMeta>, String> {
    let request = AiRequest::ListAiConversations {
        limit,
        before: before.map(str::to_string),
    };
    match call(engine, request).await? {
        AiResponse::Conversations(items) => Ok(items),
        other => panic!("réponse inattendue : {other:?}"),
    }
}

/// Lit une conversation complète, ou rend le code d'erreur
pub async fn get(engine: &Engine, id: &str) -> Result<AiConversation, String> {
    let request = AiRequest::GetAiConversation { id: id.to_string() };
    match call(engine, request).await? {
        AiResponse::Conversation(found) => Ok(found),
        other => panic!("réponse inattendue : {other:?}"),
    }
}
