#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Mémoire de l'IA par la façade : retenir, chercher, modifier, oublier, tout oublier, refuser un secret.

mod local_chat_support;
mod local_support;
mod support_ai_memory_api;
mod support_api;

use local_chat_support::fixture::LocalFixture;
use mira_core::api::{AiRequest, AiResponse};
use mira_vocabulary::{MemoryKind, MemorySource};
use support_ai_memory_api::{ask, enable, listed, remember, searched};

/// Retenir puis chercher : le souvenir revient avec un score de 1000 et son origine
#[tokio::test]
async fn remembered_text_is_found_by_search() {
    let f = LocalFixture::start_with(300, true);
    enable(&f).await;
    let record = remember(&f, "chat noir").await;
    assert_eq!(record.source, MemorySource::User);
    assert_eq!(listed(&f).await, 1);
    let found = searched(&f, "chat").await;
    assert_eq!(
        (found[0].record.id.as_str(), found[0].score_permille),
        (record.id.as_str(), 1000)
    );
}

/// Modifier remplace le texte sans changer l'identifiant ; oublier le retire de la liste
#[tokio::test]
async fn update_keeps_the_id_and_forget_removes_it() {
    let f = LocalFixture::start_with(300, true);
    enable(&f).await;
    let record = remember(&f, "chat noir").await;
    let update = AiRequest::UpdateMemory {
        id: record.id.clone(),
        text: "chien gris".into(),
    };
    let AiResponse::Memory(changed) = ask(&f, update).await.unwrap() else {
        panic!()
    };
    assert_eq!(
        (changed.id.as_str(), changed.text.as_str()),
        (record.id.as_str(), "chien gris")
    );
    ask(&f, AiRequest::ForgetMemory { id: record.id })
        .await
        .unwrap();
    assert_eq!(listed(&f).await, 0);
}

/// Tout oublier rend le nombre de souvenirs effacés
#[tokio::test]
async fn forget_all_returns_the_count() {
    let f = LocalFixture::start_with(300, true);
    enable(&f).await;
    remember(&f, "chat").await;
    remember(&f, "chien").await;
    let answer = ask(&f, AiRequest::ForgetAllMemories).await.unwrap();
    assert_eq!(answer, AiResponse::MemoryForgotten(2));
}

/// Un texte qui contient un secret est refusé et rien n'est enregistré
#[tokio::test]
async fn remembering_a_secret_is_refused() {
    let f = LocalFixture::start_with(300, true);
    enable(&f).await;
    let request = AiRequest::RememberMemory {
        text: "ma clé : sk-ant-api03-AbCdEfGhIjKlMnOpQrStUvWxYz0123456789".into(),
        kind: MemoryKind::Fact,
    };
    let error = ask(&f, request).await.unwrap_err();
    assert_eq!(error.code, "invalid_input.ai_memory_sensitive");
    assert_eq!(listed(&f).await, 0);
}
