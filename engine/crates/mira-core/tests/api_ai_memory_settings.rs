#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Réglages de la mémoire par la façade : statut initial, validation, arrêt du serveur d'embeddings à la désactivation.

mod local_chat_support;
mod local_support;
mod support_api;

use local_chat_support::fixture::LocalFixture;
use local_support::runtime::RuntimeFixture;
use mira_core::api::{AiRequest, AiResponse, Request, Response};
use mira_vocabulary::{AiConnectionId, MemoryEmbedder, MemoryKind, MemorySettings, MiraError};

/// Règle les réglages et rend le statut, ou l'erreur
async fn set(f: &LocalFixture, settings: MemorySettings) -> Result<AiResponse, MiraError> {
    let request = Request::Ai(AiRequest::SetMemorySettings { settings });
    match f.engine.execute(request).await? {
        Response::Ai(answer) => Ok(answer),
        other => panic!("réponse inattendue : {other:?}"),
    }
}

/// Un profil neuf : désactivée, calcul sur l'appareil, modèle présent, aucun souvenir
#[tokio::test]
async fn status_of_a_new_profile() {
    let f = LocalFixture::start_with(300, true);
    let answer = f
        .engine
        .execute(Request::Ai(AiRequest::GetMemoryStatus))
        .await
        .unwrap();
    let Response::Ai(AiResponse::MemoryStatus(status)) = answer else {
        panic!()
    };
    assert_eq!(status.settings, MemorySettings::default());
    assert!(status.model_installed);
    assert_eq!((status.count, status.stale), (0, 0));
}

/// Sans le modèle posé, le statut dit qu'il n'est pas installé
#[tokio::test]
async fn status_reports_a_missing_local_model() {
    let f = LocalFixture::start_with(300, false);
    let answer = f
        .engine
        .execute(Request::Ai(AiRequest::GetMemoryStatus))
        .await
        .unwrap();
    let Response::Ai(AiResponse::MemoryStatus(status)) = answer else {
        panic!()
    };
    assert!(!status.model_installed);
}

/// Une connexion inconnue est refusée
#[tokio::test]
async fn unknown_provider_connection_is_refused() {
    let f = LocalFixture::start_with(300, true);
    let unknown = MemoryEmbedder::Provider {
        connection: AiConnectionId::from_raw("inconnue"),
        model: "embed-1".into(),
    };
    let settings = MemorySettings {
        enabled: true,
        embedder: unknown,
    };
    let error = set(&f, settings).await.unwrap_err();
    assert_eq!(error.code, "not_found.ai_connection");
}

/// Un nom de modèle vide est refusé
#[tokio::test]
async fn empty_provider_model_is_refused() {
    let f = LocalFixture::start_with(300, true);
    f.add_connection(Some("mini"));
    let empty = MemoryEmbedder::Provider {
        connection: AiConnectionId::from_raw("local-1"),
        model: "  ".into(),
    };
    let error = set(
        &f,
        MemorySettings {
            enabled: true,
            embedder: empty,
        },
    )
    .await
    .unwrap_err();
    assert_eq!(error.code, "invalid_input.ai_memory_model");
}

/// Désactiver la mémoire arrête le serveur d'embeddings
#[tokio::test]
async fn disabling_stops_the_embedding_server() {
    let f = LocalFixture::start_with(300, true);
    let on = MemorySettings {
        enabled: true,
        embedder: MemoryEmbedder::Local,
    };
    set(&f, on).await.unwrap();
    let store = f.engine.memory_store().await.unwrap();
    store
        .remember(
            "chat",
            MemoryKind::Fact,
            mira_vocabulary::MemorySource::User,
        )
        .await
        .unwrap();
    let pid = f.entries("start")[0]["pid"].as_u64().unwrap();
    assert!(RuntimeFixture::alive(pid));
    set(&f, MemorySettings::default()).await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    assert!(!RuntimeFixture::alive(pid));
}
