#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Mémoire côté moteur, mode local : désactivée sans lancement, modèle requis, serveur dédié lancé une fois, arrêt à la fermeture.

mod local_chat_support;
mod local_support;
mod support_api;

use std::time::Duration;

use local_chat_support::fixture::LocalFixture;
use local_support::runtime::RuntimeFixture;
use mira_vocabulary::error::codes::incompatible_state::{
    AI_LOCAL_UNSUPPORTED_PLATFORM, AI_MEMORY_DISABLED, AI_MODEL_MISSING,
};
use mira_vocabulary::{MemoryEmbedder, MemoryKind, MemorySettings, MemorySource};

/// Active la mémoire en mode local dans la base du profil
fn enable_local(f: &LocalFixture) {
    let settings = MemorySettings {
        enabled: true,
        embedder: MemoryEmbedder::Local,
    };
    f.db.write(|tx| tx.set_ai_memory_settings(&settings))
        .unwrap();
}

/// Mémoire désactivée : erreur typée et aucun serveur lancé
#[tokio::test]
async fn disabled_memory_is_a_typed_error_and_launches_nothing() {
    let f = LocalFixture::start_with(300, true);
    let error = f.engine.memory_store().await.unwrap_err();
    assert_eq!(error.code, AI_MEMORY_DISABLED.0);
    assert_eq!(f.starts(), 0);
}

/// Mode local, modèle déclaré mais fichier absent : modèle manquant, rien n'est lancé
#[tokio::test]
async fn local_without_installed_model_is_model_missing() {
    let f = LocalFixture::start_with(300, true);
    let file = f
        .root
        .parent()
        .unwrap()
        .join("ai-memory/embeddings/embed.bin");
    std::fs::remove_file(file).unwrap();
    enable_local(&f);
    let error = f.engine.memory_store().await.unwrap_err();
    assert_eq!(error.code, AI_MODEL_MISSING.0);
    assert_eq!(f.starts(), 0);
}

/// Mode local, fiche sans modèle d'embeddings : plateforme non prise en charge
#[tokio::test]
async fn local_without_embedding_in_the_runtime_is_unsupported() {
    let f = LocalFixture::start_with(300, false);
    enable_local(&f);
    let error = f.engine.memory_store().await.unwrap_err();
    assert_eq!(error.code, AI_LOCAL_UNSUPPORTED_PLATFORM.0);
}

/// Le serveur d'embeddings est lancé une seule fois, en mode embeddings, pour retenir puis retrouver
#[tokio::test]
async fn local_mode_embeds_through_one_dedicated_server() {
    let f = LocalFixture::start_with(300, true);
    enable_local(&f);
    let store = f.engine.memory_store().await.unwrap();
    store
        .remember("chat noir", MemoryKind::Fact, MemorySource::User)
        .await
        .unwrap();
    let again = f.engine.memory_store().await.unwrap();
    let hits = again.recall("chat", 5).await.unwrap();
    assert_eq!(hits[0].record.text, "chat noir");
    assert_eq!(f.starts(), 1);
    let argv = f.entries("start")[0]["argv"].to_string();
    assert!(argv.contains("--embedding") && argv.contains("cls"));
}

/// La fermeture du moteur arrête le serveur d'embeddings
#[tokio::test]
async fn shutdown_stops_the_embedding_server() {
    let f = LocalFixture::start_with(300, true);
    enable_local(&f);
    let store = f.engine.memory_store().await.unwrap();
    store
        .remember("chien", MemoryKind::Fact, MemorySource::User)
        .await
        .unwrap();
    let pid = f.entries("start")[0]["pid"].as_u64().unwrap();
    assert!(RuntimeFixture::alive(pid));
    drop(store);
    let LocalFixture { engine, .. } = f;
    let engine = std::sync::Arc::try_unwrap(engine).unwrap();
    engine
        .shutdown_within(Duration::from_secs(2))
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert!(!RuntimeFixture::alive(pid));
}
