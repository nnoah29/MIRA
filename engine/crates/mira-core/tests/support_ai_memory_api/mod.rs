#![allow(dead_code, clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Raccourcis des tests de la mémoire par la façade : requête, activation, ajout, comptes et recherche.

use crate::local_chat_support::fixture::LocalFixture;
use mira_core::api::{AiRequest, AiResponse, Request, Response};
use mira_vocabulary::{
    MemoryEmbedder, MemoryKind, MemoryMatch, MemoryRecord, MemorySettings, MiraError,
};

/// Exécute une requête de la mémoire et rend la réponse du domaine IA
pub async fn ask(f: &LocalFixture, request: AiRequest) -> Result<AiResponse, MiraError> {
    match f.engine.execute(Request::Ai(request)).await? {
        Response::Ai(answer) => Ok(answer),
        other => panic!("réponse inattendue : {other:?}"),
    }
}

/// Active la mémoire en mode local par la façade
pub async fn enable(f: &LocalFixture) {
    let settings = MemorySettings {
        enabled: true,
        embedder: MemoryEmbedder::Local,
    };
    ask(f, AiRequest::SetMemorySettings { settings })
        .await
        .unwrap();
}

/// Retient ce texte comme un fait et rend le souvenir
pub async fn remember(f: &LocalFixture, text: &str) -> MemoryRecord {
    let request = AiRequest::RememberMemory {
        text: text.to_string(),
        kind: MemoryKind::Fact,
    };
    match ask(f, request).await.unwrap() {
        AiResponse::Memory(record) => record,
        other => panic!("réponse inattendue : {other:?}"),
    }
}

/// Nombre de souvenirs listés
pub async fn listed(f: &LocalFixture) -> usize {
    match ask(
        f,
        AiRequest::ListMemories {
            limit: 50,
            after: None,
        },
    )
    .await
    .unwrap()
    {
        AiResponse::Memories(items) => items.len(),
        other => panic!("réponse inattendue : {other:?}"),
    }
}

/// Résultats de la recherche de ce texte
pub async fn searched(f: &LocalFixture, query: &str) -> Vec<MemoryMatch> {
    match ask(
        f,
        AiRequest::SearchMemories {
            query: query.into(),
            limit: 5,
        },
    )
    .await
    .unwrap()
    {
        AiResponse::MemoryMatches(found) => found,
        other => panic!("réponse inattendue : {other:?}"),
    }
}
